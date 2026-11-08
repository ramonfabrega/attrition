//! `MountainRange::validate_tcoord@0089ad00` under the emulator against the port (item 2149; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `MountainRange::validate_tcoord@0089ad00`, run by `tools/emu/sweep_obj.py` on its row `mountain_range_validate_tcoord` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 32 random bytes of object, 5 argument(s), argument 0 names 128 random bytes; argument 3 names 16 random bytes; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("mountain_range_validate_tcoord") else {
        return;
    };
    let obj = super::Obj {
        name: "validate_tcoord",
        full: "MountainRange::validate_tcoord@0089ad00",
        cells: 0,
        calls: false,
        eax: true,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, _log| {
        let _ = (&a, &m, &c, &p);
        {
            let i = (a[2] * a[4] + a[1]) as usize;
            let j = ((a[2] + 1) * a[4] + a[1]) as usize;
            let v = [
                rd(&p[0], 4 * i),
                rd(&p[0], 4 * i + 4),
                rd(&p[0], 4 * j),
                rd(&p[0], 4 * j + 4),
            ];
            for (k, x) in v.iter().enumerate() {
                wr(&mut p[1], 4 * k, *x);
            }
            i64::from(v.contains(&-1))
        }
    });
}
