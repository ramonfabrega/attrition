//! `World::is_edge_of_region@006b3180` under the emulator against the port (item 2743; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `World::is_edge_of_region@006b3180`, run by `tools/emu/sweep_obj.py` on its row `world_is_edge_of_region` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 16 random bytes of object, 4 argument(s), 32 random bytes at 0x00adc404; 32 random bytes at 0x00adcaf4; argument 0 names 4 random bytes; argument 1 names 4 random bytes; the word at `0xc06188` points at 320 random bytes less `0` (placed); the word at `p2+0x134` points at 448 random bytes less `0` (placed); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("world_is_edge_of_region") else {
        return;
    };
    let obj = super::Obj {
        name: "is_edge_of_region",
        full: "World::is_edge_of_region@006b3180",
        cells: 2,
        calls: false,
        eax: true,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, _log| {
        let _ = (&a, &m, &c, &p);
        // A tile is on the edge of its region when any of the eight neighbours (`dx` at `0xadcaf4`, `dy` at
        // `0xadc404`) that lies on the map has a different region short (tile +4, 28-byte tiles).
        let (x, y) = (rd(&p[0], 0), rd(&p[1], 0));
        let (w, h) = (rd(&p[2], 0), rd(&p[2], 4));
        let region = |tx: i32, ty: i32| -> u16 {
            let at = (w.wrapping_mul(ty).wrapping_add(tx) as usize) * 28 + 4;
            u16::from_le_bytes([p[3][at], p[3][at + 1]])
        };
        let centre = region(x, y);
        let edge = (0..8).any(|k| {
            let (nx, ny) = (
                rd(&c[1], 4 * k).wrapping_add(x),
                rd(&c[0], 4 * k).wrapping_add(y),
            );
            nx >= 0 && ny >= 0 && nx < w && ny < h && region(nx, ny) != centre
        });
        i64::from(edge)
    });
}
