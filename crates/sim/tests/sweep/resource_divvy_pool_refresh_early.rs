//! `ResourceDivvyPool::refresh_early@0068a480` under the emulator against the port (item 2181; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `ResourceDivvyPool::refresh_early@0068a480`, run by `tools/emu/sweep_obj.py` on its row `resource_divvy_pool_refresh_early` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 12 random bytes of object, 0 argument(s), the word at `m+0x8` points at 64 random bytes less `0` (placed); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("resource_divvy_pool_refresh_early") else {
        return;
    };
    let obj = super::Obj {
        name: "refresh_early",
        full: "ResourceDivvyPool::refresh_early@0068a480",
        cells: 0,
        calls: false,
        eax: false,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, _log| {
        let _ = (&a, &m, &c, &p);
        {
            let n = rd(m, 0x4);
            // `0 < size` gates the clear; a size past the buffer is not drawn.
            p[0][..n.max(0) as usize].fill(0);
            0
        }
    });
}
