//! `PathFinder::valid_wcoord@00687da0` under the emulator against the port (item 2200; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `PathFinder::valid_wcoord@00687da0`, run by `tools/emu/sweep_obj.py` on its row `path_finder_valid_wcoord` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 88 random bytes of object, 5 argument(s), the global cell 0x00cae5fc points at 1024 random bytes; callee 0x00607c30 (8 args) stubbed; the word at `m+0x54` points at 296 random bytes less `0` (placed); stub 0 answered from [0, 1, 2, 3, 3, 4, -1] (90%); argument 0 copies `p0+0x120` (45%); argument 1 copies `p0+0x124` (45%); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("path_finder_valid_wcoord") else {
        return;
    };
    let obj = super::Obj {
        name: "valid_wcoord",
        full: "PathFinder::valid_wcoord@00687da0",
        cells: 1,
        calls: false,
        eax: true,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, _log| {
        let _ = (&a, &m, &c, &p);
        {
            if a[0] == rd(&p[0], 0x120) && a[1] == rd(&p[0], 0x124) {
                0
            } else {
                i64::from(crate::movement::path::wcoord_verdict(
                    a[5],
                    crate::Pos::new(a[0], a[1]),
                    crate::Pos::new(a[3], a[4]),
                ))
            }
        }
    });
}
