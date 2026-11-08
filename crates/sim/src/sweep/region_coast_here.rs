//! `Region::coast_here@00681020` under the emulator against the port (item 2200; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `Region::coast_here@00681020`, run by `tools/emu/sweep_obj.py` on its row `region_coast_here` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 20 random bytes of object, 3 argument(s), the global cell 0x00c06188 points at 312 random bytes; callee 0x00680f90 (1 args) stubbed; callee 0x006b52e0 (2 args) stubbed; the word at `c0+0x134` points at 4 random bytes less `(i32(c[0],0)*a[2]+a[1])*0x1c+4` (placed); stub 0 answered from [0, 1, 1, 1] (90%); stub 1 answered from [1, 65, -1] (90%); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("region_coast_here") else {
        return;
    };
    let obj = super::Obj {
        name: "coast_here",
        full: "Region::coast_here@00681020",
        cells: 1,
        calls: false,
        eax: true,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, _log| {
        let _ = (&a, &m, &c, &p);
        {
            if a[3] == 0 {
                0
            } else {
                i64::from(crate::world::coast_scan(
                    crate::world::Cell::new(a[1], a[2]),
                    (rd(&c[0], 0), rd(&c[0], 4)),
                    (rd(m, 0x10), a[0]),
                    i32::from(i16::from_le_bytes([p[0][0], p[0][1]])),
                    |_| a[4],
                ))
            }
        }
    });
}
