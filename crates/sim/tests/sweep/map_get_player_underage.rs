//! `Map::get_player_underage@0068b2f0` under the emulator against the port (item 2223; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `Map::get_player_underage@0068b2f0`, run by `tools/emu/sweep_obj.py` on its row `map_get_player_underage` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 8 random bytes of object, 1 argument(s), the global cell 0x00c061ec points at 40 random bytes; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("map_get_player_underage") else {
        return;
    };
    let obj = super::Obj {
        name: "get_player_underage",
        full: "Map::get_player_underage@0068b2f0",
        cells: 1,
        calls: false,
        eax: true,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, _log| {
        let _ = (&a, &m, &c, &p);
        {
            let base = match c[0][0x26] {
                0..=2 => 2,
                3 => 4,
                4 => 5,
                5 => 7,
                _ => 8,
            };
            let v = base - a[0];
            if v < 0 || a[0] < 0 {
                0
            } else {
                i64::from(v.min(a[0]))
            }
        }
    });
}
