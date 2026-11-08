//! `RiverSectionOut::compute_vert_height@0088b6a0` under the emulator against the port (item 2807; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `RiverSectionOut::compute_vert_height@0088b6a0`, run by `tools/emu/sweep_obj.py` on its row `river_section_out_compute_vert_height` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 16 random bytes of object, 1 argument(s), 8 random bytes at 0x00c11cc8; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("river_section_out_compute_vert_height") else {
        return;
    };
    let obj = super::Obj {
        name: "compute_vert_height",
        full: "RiverSectionOut::compute_vert_height@0088b6a0",
        cells: 1,
        calls: false,
        eax: true,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, _log| {
        let _ = (&a, &m, &c, &p);
        {
            // The nine-float table (1.0, 0.75, 0.26, 0.0 | 0.0, 0.0, 0.26, 0.75 | 1.0; the image's 0xb69870, 0xb69ac0 and
            // a literal 1.0) at the argument; c is the float at 0xc11ccc, b the byte at 0xc11cc8, and the 0.25 at
            // 0xb69484: c - t*c + (c*0.25) * -b, each step a single-precision operation in the order written.
            const T: [u32; 9] = [
                0x3f800000, 0x3f400000, 0x3e851eb8, 0, 0, 0, 0x3e851eb8, 0x3f400000, 0x3f800000,
            ];
            let t = f32::from_bits(T[a[0] as usize]);
            let cf = f32::from_bits(u32::from_le_bytes([c[0][4], c[0][5], c[0][6], c[0][7]]));
            let b = i32::from(c[0][0]);
            let r = (cf - t * cf) + (cf * 0.25) * ((-b) as f32);
            i64::from(r.to_bits() as i32)
        }
    });
}
