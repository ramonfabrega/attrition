//! `BorderNodes::find@00882170` under the emulator against the port (item 2633; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `BorderNodes::find@00882170`, run by `tools/emu/sweep_obj.py` on its row `border_nodes_find` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 32 random bytes of object, 1 argument(s), the word at `m+0x10` points at 16 random bytes less `0` (placed); the word at `p0+0` points at 16 random bytes less `0` (placed); the word at `p0+4` points at 16 random bytes less `0` (placed); the word at `p0+8` points at 16 random bytes less `0` (placed); the word at `p0+0xc` points at 16 random bytes less `0` (placed); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("border_nodes_find") else {
        return;
    };
    let obj = super::Obj {
        name: "find",
        full: "BorderNodes::find@00882170",
        cells: 0,
        calls: false,
        eax: true,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, _log| {
        let _ = (&a, &m, &c, &p);
        let n = rd(m, 4).max(0) as usize;
        let mut r = -1;
        for i in 0..n {
            let rec = &p[1 + i];
            if a[0] == i32::from(rec[10]) && (rd(rec, 4) == 0 || rd(rec, 0) == 0) {
                r = i as i32;
                break;
            }
        }
        i64::from(r)
    });
}
