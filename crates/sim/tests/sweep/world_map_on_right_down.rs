//! `WorldMap::on_right_down@008c7000` under the emulator against the port (item 2807; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `WorldMap::on_right_down@008c7000`, run by `tools/emu/sweep_obj.py` on its row `world_map_on_right_down` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 4920 random bytes of object, 2 argument(s), 8 random bytes at 0x00e334f0; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("world_map_on_right_down") else {
        return;
    };
    let obj = super::Obj {
        name: "on_right_down",
        full: "WorldMap::on_right_down@008c7000",
        cells: 1,
        calls: false,
        eax: false,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, _log| {
        let _ = (&a, &m, &c, &p);
        {
            // Dragging is set; with no mouse capture held (the word at 0xe334f4, c[0]+4) the window takes it —
            // its own address, which the emulator lays at 0x20000000 — and clears the non-client capture
            // (0xe334f0); the press point is stored twice.
            wr(m, 0x1324, 1);
            if rd(&c[0], 4) == 0 {
                wr(&mut c[0], 4, 0x2000_0000);
                wr(&mut c[0], 0, 0);
            }
            wr(m, 0x1328, a[0]);
            wr(m, 0x132c, a[1]);
            wr(m, 0x1330, a[0]);
            wr(m, 0x1334, a[1]);
            0
        }
    });
}
