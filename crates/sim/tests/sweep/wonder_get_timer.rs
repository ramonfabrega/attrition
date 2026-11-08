//! `Wonder::get_timer@0073c660` under the emulator against the port (item 2281; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `Wonder::get_timer@0073c660`, run by `tools/emu/sweep_obj.py` on its row `wonder_get_timer` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 16 random bytes of object, 0 argument(s), the global cell 0x00c061f0 points at 3344 random bytes; the global cell 0x00c06188 points at 4 random bytes; the global cell 0x00e7fca8 points at 328 random bytes; the global cell 0x00c0618c points at 224 random bytes; the word at `c3+0x14+i8(m,0xd)*0x1c` points at 4 random bytes less `i16(m,2)*4` (placed); the word at `p0+0` points at 4 random bytes less `0x18` (placed); the word at `p1+0` points at 4 random bytes less `4` (placed); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("wonder_get_timer") else {
        return;
    };
    let obj = super::Obj {
        name: "get_timer",
        full: "Wonder::get_timer@0073c660",
        cells: 4,
        calls: false,
        eax: true,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, _log| {
        let _ = (&a, &m, &c, &p);
        {
            // Each of the two constants scaled by the world's size over the map
            // size's `+0x144`, rounded, at least 1; the wonder's type against 0x21e.
            let size = rd(&c[2], 0x144);
            let world = rd(&c[1], 0);
            let scaled = |k: i32| -> i32 {
                if k < 1 {
                    0
                } else {
                    (size / 2)
                        .wrapping_add(world.wrapping_mul(k))
                        .wrapping_div(size)
                        .max(1)
                }
            };
            let timer = scaled(rd(&c[0], 0xd08));
            let age = scaled(rd(&c[0], 0xd0c));
            i64::from(
                0x21e_i32
                    .wrapping_sub(rd(&p[2], 0))
                    .wrapping_mul(age)
                    .wrapping_add(timer),
            )
        }
    });
}
