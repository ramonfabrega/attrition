//! `CollBlock::set@00682070` under the emulator against the port (item 2217; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `CollBlock::set@00682070`, run by `tools/emu/sweep_obj.py` on its row `coll_block_set` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 44 random bytes of object, 3 argument(s), no windows; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("coll_block_set") else {
        return;
    };
    let obj = super::Obj {
        name: "set",
        full: "CollBlock::set@00682070",
        cells: 0,
        calls: false,
        eax: false,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, _log| {
        let _ = (&a, &m, &c, &p);
        {
            // The block's 16 x 16 bits at `+0xc`, x-major (`(x % 16) * 16 + y % 16`),
            // laid into the port's flattened grid over one world cell and read back.
            let n = crate::collide::UCELLS_PER_CELL;
            let mut g = crate::collide::CollGrid::new(1, 1);
            for i in 0..(n * n) as usize {
                if m[0xc + i / 8] >> (i % 8) & 1 != 0 {
                    g.set(i as i32 / n, i as i32 % n, true);
                }
            }
            g.set(a[0] % n, a[1] % n, a[2] != 0);
            for i in 0..(n * n) as usize {
                let bit = 1u8 << (i % 8);
                if g.get(i as i32 / n, i as i32 % n) {
                    m[0xc + i / 8] |= bit;
                } else {
                    m[0xc + i / 8] &= !bit;
                }
            }
            // `+8`, the block's state: 0 once a bit is set, 2 on a clear of a clean block.
            if a[2] != 0 {
                wr(m, 8, 0);
            } else if rd(m, 8) == 0 {
                wr(m, 8, 2);
            }
            0
        }
    });
}
