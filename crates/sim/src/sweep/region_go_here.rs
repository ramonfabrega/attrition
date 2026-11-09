//! `Region::go_here@006810f0` under the emulator against the port (item 2287; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `Region::go_here@006810f0`, run by `tools/emu/sweep_obj.py` on its row `region_go_here` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 32 random bytes of object, 1 argument(s), 8 rows 0x6eec apart at `@0xe3a390`, slices +0x0:0x4, +0x8:0x4, +0x74:0x20, +0xa68:0x10, +0xfde:0x10, +0x125e:0x10 (`p` windows after the pointed buffers, before the places); 8 rows 0x6eec apart at `0xc061e0`, slices +0x74:0x20 (`p` windows after the pointed buffers, before the places); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("region_go_here") else {
        return;
    };
    let obj = super::Obj {
        name: "go_here",
        full: "Region::go_here@006810f0",
        cells: 0,
        calls: false,
        eax: true,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, _log| {
        let _ = (&a, &m, &c, &p);
        // The region `+8` flags, `+0x10` its index, `+0x14` its size;
        // leader i's slices `p[6i..6i+6]` — byte 0, `+8` (its team),
        // `+0x74` the stances, `+0xa68` the strategy cells, `+0xfde`
        // the peasants, `+0x125e` the cities — and `leadersc`'s stances
        // `p[48 + i]`. `a[0]` is `who`.
        let who = a[0] as usize;
        let r = rd(m, 0x10) as usize;
        let short = |w: &[u8], k: usize| i32::from(u16::from_le_bytes([w[2 * k], w[2 * k + 1]]));
        let team = rd(&p[6 * who + 1], 0);
        i64::from(crate::ai::army::go_here_bits(
            m[8] & 8 != 0,
            rd(m, 0x14),
            who,
            short(&p[6 * who + 5], r),
            short(&p[6 * who + 4], r),
            p[6 * who + 3][2 * r] & 4 != 0,
            8,
            |i| p[6 * i][0] & 2 != 0,
            |i| short(&p[6 * i + 5], r),
            |i| {
                i as i32 != team
                    && (rd(&p[6 * who + 2], 4 * i) == 0 || rd(&p[48 + i], 4 * team as usize) == 0)
            },
        ))
    });
}
