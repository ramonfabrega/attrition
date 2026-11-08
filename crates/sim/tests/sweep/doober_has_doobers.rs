//! `Doober::has_doobers@008466d0` under the emulator against the port (item 2783; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `Doober::has_doobers@008466d0`, run by `tools/emu/sweep_obj.py` on its row `doober_has_doobers` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 16 random bytes of object, 2 argument(s), 4 random bytes at 0x00eafdfc; the word at `0xeafe08` points at 64 random bytes less `0` (placed); the word at `0xcae5fc` points at 16 random bytes less `0` (placed); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("doober_has_doobers") else {
        return;
    };
    let obj = super::Obj {
        name: "has_doobers",
        full: "Doober::has_doobers@008466d0",
        cells: 1,
        calls: false,
        eax: true,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, _log| {
        let _ = (&a, &m, &c, &p);
        // `Doober::has_doobers(p1, p2)`: over the `count` (`c[0]`) float pairs of the locations table (`p[0]`)
        // it truncates the x float, takes `>> 8` as an index into `div_3_table` (`p[1]`) and, on a row
        // equal to `p1`, answers 1 if `div_3_table[0]` is nonzero: the original's second test indexes the
        // table by `(int)(float)(y == (float)p2) >> 8`, which is 0 whatever the comparison said, so `p2`
        // and `y` are never decisive.
        let count = rd(&c[0], 0);
        let table = |i: i32| rd(&p[1], 4 * i as usize);
        let mut ans = 0i64;
        for i in 0..count.max(0) as usize {
            let x = f32::from_bits(rd(&p[0], 8 * i) as u32) as i32;
            if table(x >> 8) == a[0] && table(0) != 0 {
                ans = 1;
                break;
            }
        }
        ans
    });
}
