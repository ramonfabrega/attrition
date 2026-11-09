//! `Armies::find_aggressive_army@006f2e10` under the emulator against the port (item 2247; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `Armies::find_aggressive_army@006f2e10`, run by `tools/emu/sweep_obj.py` on its row `armies_find_aggressive_army` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 32 random bytes of object, 1 argument(s), the global cell 0x00cae5fc points at 1024 random bytes; the global cell 0x00c06188 points at 320 random bytes; the word at `0xc09710+a[0]*0x1c` points at 64 random bytes less `0` (placed); the word at `p0+0` points at 64 random bytes less `0` (placed); the word at `p0+4` points at 64 random bytes less `0` (placed); the word at `p0+8` points at 64 random bytes less `0` (placed); the word at `p0+12` points at 64 random bytes less `0` (placed); the word at `p0+16` points at 64 random bytes less `0` (placed); the word at `p0+20` points at 64 random bytes less `0` (placed); the word at `p0+24` points at 64 random bytes less `0` (placed); the word at `p0+28` points at 64 random bytes less `0` (placed); the word at `p0+32` points at 64 random bytes less `0` (placed); the word at `p0+36` points at 64 random bytes less `0` (placed); the word at `p0+40` points at 64 random bytes less `0` (placed); the word at `p0+44` points at 64 random bytes less `0` (placed); the word at `p0+48` points at 64 random bytes less `0` (placed); the word at `p0+52` points at 64 random bytes less `0` (placed); the word at `p0+56` points at 64 random bytes less `0` (placed); the word at `p0+60` points at 64 random bytes less `0` (placed); the word at `c1+0x134` points at 1792 random bytes less `0` (placed); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("armies_find_aggressive_army") else {
        return;
    };
    let obj = super::Obj {
        name: "find_aggressive_army",
        full: "Armies::find_aggressive_army@006f2e10",
        cells: 2,
        calls: false,
        eax: true,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, _log| {
        let _ = (&a, &m, &c, &p);
        {
            // `who` is `a[0]`; its sixteen armies are `p[1..17]` (`+0` live,
            // `+4` status, `+0x14` captains, `+0x38`/`+0x3c` the position),
            // the world `c[1]` and its cell records `p[17]`.
            let cw = rd(&c[1], 0);
            let found = (0..16).find(|&k| {
                let r = &p[1 + k];
                let (x, y) = (
                    rd(&c[0], 4 * (rd(r, 0x38) >> 8) as usize),
                    rd(&c[0], 4 * (rd(r, 0x3c) >> 8) as usize),
                );
                let owner = i32::from(p[17][((y * cw + x) * 0x1c + 0xf) as usize] as i8);
                let live = i16::from_le_bytes([r[0], r[1]]) != 0;
                crate::ai::army::aggressive(live, rd(r, 0x14), i32::from(r[4]), owner, a[0])
            });
            found.map_or(-1, |k| k as i64)
        }
    });
}
