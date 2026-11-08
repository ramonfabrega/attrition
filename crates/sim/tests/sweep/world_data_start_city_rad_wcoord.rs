//! `WorldData::start_city_rad_wcoord@006b3850` under the emulator against the port (item 3008; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `WorldData::start_city_rad_wcoord@006b3850`, run by `tools/emu/sweep_obj.py` on its row `world_data_start_city_rad_wcoord` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 8 random bytes of object, 2 argument(s), `p` windows in order: p0..p1 pointed argument(s), p2..p3 the place(s), in flag order; the global cell 0x00c06188 points at 1024 random bytes; the global cell 0x00c061e4 points at 512 random bytes; argument 0 names 4 random bytes; argument 1 names 4 random bytes; the word at `c0+0xc8` points at 64 random bytes less `0` (placed); the word at `c0+0xe4` points at 64 random bytes less `0` (placed); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 6000 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("world_data_start_city_rad_wcoord") else {
        return;
    };
    let obj = super::Obj {
        name: "start_city_rad_wcoord",
        full: "WorldData::start_city_rad_wcoord@006b3850",
        cells: 2,
        calls: false,
        eax: true,
        min_rows: 6000,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, _log| {
        let _ = (&a, &m, &c, &p);
        {
            let n = rd(&c[0], 0xbc);
            let (px, py) = (rd(&p[0], 0), rd(&p[1], 0));
            let radius = rd(&c[1], 0x12c);
            let approx = |dx: u32, dy: u32| -> u32 {
                let (hi, lo) = if dy < dx { (dx, dy) } else { (dy, dx) };
                if hi == 0 {
                    0
                } else if lo < 60000 {
                    (lo * lo) / (hi * 2) + hi
                } else {
                    (lo + hi * 2) >> 1
                }
            };
            let mut found = 0;
            for k in 0..n.max(0) as usize {
                let dx = rd(&p[2], 4 * k).wrapping_sub(px).unsigned_abs();
                let dy = rd(&p[3], 4 * k).wrapping_sub(py).unsigned_abs();
                if (approx(dx, dy).wrapping_mul(4) as i32) < radius - 1 {
                    found = 1;
                    break;
                }
            }
            found
        }
    });
}
