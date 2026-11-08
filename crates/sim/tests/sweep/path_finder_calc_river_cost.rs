//! `PathFinder::calc_river_cost@00687930` under the emulator against the port (item 2241; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `PathFinder::calc_river_cost@00687930`, run by `tools/emu/sweep_obj.py` on its row `path_finder_calc_river_cost` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 8 random bytes of object, 3 argument(s), callee 0x00a39d70 (2 args) stubbed; stub 0 answered from [0, 1, 9, 10, 19, 20, 21, 12345, 65535, -1, -21] (90%); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("path_finder_calc_river_cost") else {
        return;
    };
    let obj = super::Obj {
        name: "calc_river_cost",
        full: "PathFinder::calc_river_cost@00687930",
        cells: 0,
        calls: false,
        eax: true,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, _log| {
        let _ = (&a, &m, &c, &p);
        {
            if a[2] < 0 {
                0
            } else {
                i64::from(a[3] % 20 + 10)
            }
        }
    });
}
