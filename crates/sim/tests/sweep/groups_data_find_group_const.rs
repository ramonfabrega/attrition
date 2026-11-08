//! `GroupsData::find_group_const@007084c0` under the emulator against the port (item 2743; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `GroupsData::find_group_const@007084c0`, run by `tools/emu/sweep_obj.py` on its row `groups_data_find_group_const` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 16 random bytes of object, 3 argument(s), the word at `0xe85f20` points at 276760 random bytes less `0` (placed); the word at `0xc0ab84` points at 12 random bytes less `0` (placed); the word at `0xc0aba0` points at 12 random bytes less `0` (placed); the word at `p1+0` points at 16 random bytes less `0` (placed); the word at `p1+4` points at 16 random bytes less `0` (placed); the word at `p1+8` points at 16 random bytes less `0` (placed); the word at `p2+0` points at 16 random bytes less `0` (placed); the word at `p2+4` points at 16 random bytes less `0` (placed); the word at `p2+8` points at 16 random bytes less `0` (placed); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("groups_data_find_group_const") else {
        return;
    };
    let obj = super::Obj {
        name: "find_group_const",
        full: "GroupsData::find_group_const@007084c0",
        cells: 0,
        calls: false,
        eax: true,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, _log| {
        let _ = (&a, &m, &c, &p);
        // The first of the player's 46 group records (0x9d4 bytes, 64 records a player in the image's array,
        // the owner byte at +0x4a) owned by `a[0]` whose member list (`+0xc` count of shorts at `+0x8cc`)
        // holds the object `a[1]`, provided that object is live (bit 0 of its +8); the answer is the
        // record's index in the array.
        let (who, idx) = (a[0] as usize, a[1] as usize);
        let live = rd(&p[3 + 3 * who + idx], 8) & 1 != 0;
        (who << 6..(who << 6) + 46)
            .find(|&r| {
                let rec = &p[0][r * 0x9d4..(r + 1) * 0x9d4];
                if rec[0x4a] as usize != who || !live {
                    return false;
                }
                let n = rd(rec, 0xc).max(0) as usize;
                (0..n).any(|j| {
                    i32::from(i16::from_le_bytes([rec[0x8cc + 2 * j], rec[0x8cd + 2 * j]])) == a[1]
                })
            })
            .map_or(-1, |r| r as i64)
    });
}
