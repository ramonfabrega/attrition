//! `ResourceDivvyPool::add_early@0068a800` under the emulator against the port (item 2211; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `ResourceDivvyPool::add_early@0068a800`, run by `tools/emu/sweep_obj.py` on its row `resource_divvy_pool_add_early` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 32 random bytes of object, 1 argument(s), vtable slot +0x0 (1 args) stubbed; the word at `m+0x1c` points at 4 random bytes less `i32(m,0x10)*4` (placed); the stubbed vtable laid at `m+0xc`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("resource_divvy_pool_add_early") else {
        return;
    };
    let obj = super::Obj {
        name: "add_early",
        full: "ResourceDivvyPool::add_early@0068a800",
        cells: 0,
        calls: false,
        eax: false,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, _log| {
        let _ = (&a, &m, &c, &p);
        {
            wr(&mut p[0], 0, a[0]);
            let n = rd(m, 0x10).wrapping_add(1);
            wr(m, 0x10, n);
            0
        }
    });
}
