//! `Regions::concat@00680500` under the emulator against the port (item 2298; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `Regions::concat@00680500`, run by `tools/emu/sweep_obj.py` on its row `regions_concat` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 32 random bytes of object, 2 argument(s), the global cell 0x00c06188 points at 320 random bytes; the word at `m+0x10` points at 680 random bytes less `0` (placed); the word at `c0+0x134` points at 336 random bytes less `0` (placed); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("regions_concat") else {
        return;
    };
    let obj = super::Obj {
        name: "concat",
        full: "Regions::concat@00680500",
        cells: 1,
        calls: false,
        eax: false,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, _log| {
        let _ = (&a, &m, &c, &p);
        {
            // Region a[0] folds into a[1]: its +0x14 word (the regions at this+0x10,
            // 0x88 apart) is added to a[1]'s and zeroed; then every tile record of the
            // world (a count at +8, 0x1c-byte records at +0x134) whose region short
            // (+4) is a[0] is relabelled a[1].
            let (s, d) = (a[0] as usize * 0x88 + 0x14, a[1] as usize * 0x88 + 0x14);
            let sum = rd(&p[0], d).wrapping_add(rd(&p[0], s));
            wr(&mut p[0], d, sum);
            wr(&mut p[0], s, 0);
            for i in 0..rd(&c[0], 8).max(0) as usize {
                let o = i * 0x1c + 4;
                if i32::from(i16::from_le_bytes([p[1][o], p[1][o + 1]])) == a[0] {
                    p[1][o..o + 2].copy_from_slice(&(a[1] as i16).to_le_bytes());
                }
            }
            0
        }
    });
}
