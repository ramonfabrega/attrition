//! `UnbuiltForts::remove_unbuilt_fort@0073bc50` under the emulator against the port (item 2298; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `UnbuiltForts::remove_unbuilt_fort@0073bc50`, run by `tools/emu/sweep_obj.py` on its row `unbuilt_forts_remove_unbuilt_fort` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 16 random bytes of object, 2 argument(s), 224 random bytes at 0x00e39b24; the word at `c0+0xc+a[1]*0x1c` points at 36 random bytes less `0` (placed); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("unbuilt_forts_remove_unbuilt_fort") else {
        return;
    };
    let obj = super::Obj {
        name: "remove_unbuilt_fort",
        full: "UnbuiltForts::remove_unbuilt_fort@0073bc50",
        cells: 1,
        calls: false,
        eax: false,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, _log| {
        let _ = (&a, &m, &c, &p);
        {
            // The forts of `who` (a[1]): a count at +0 and a list of 4-byte
            // entries at +0xc of row `who` (stride 0x1c); the first entry whose low
            // short equals a[0] takes the last entry's word and the count drops.
            let row = a[1] as usize * 0x1c;
            let n = rd(&c[0], row);
            if let Some(i) = (0..n.max(0) as usize)
                .find(|&i| i32::from(i16::from_le_bytes([p[0][4 * i], p[0][4 * i + 1]])) == a[0])
            {
                let last = rd(&p[0], 4 * (n as usize - 1));
                wr(&mut p[0], 4 * i, last);
                wr(&mut c[0], row, n - 1);
            }
            0
        }
    });
}
