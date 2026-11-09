//! `Army::member@006f8de0` under the emulator against the port (item 2844; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `Army::member@006f8de0`, run by `tools/emu/sweep_obj.py` on its row `army_member_by_unit` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 160 random bytes of object, 2 argument(s), the word at `0xc0ab84` points at 16 random bytes less `0` (placed); the word at `p12+0` points at 12 random bytes less `0` (placed); the word at `p12+4` points at 12 random bytes less `0` (placed); the word at `p12+8` points at 12 random bytes less `0` (placed); the word at `p12+12` points at 12 random bytes less `0` (placed); the word at `0xc0aba0` points at 16 random bytes less `0` (placed); the word at `p17+0` points at 12 random bytes less `0` (placed); the word at `p17+4` points at 12 random bytes less `0` (placed); the word at `p17+8` points at 12 random bytes less `0` (placed); the word at `p17+12` points at 12 random bytes less `0` (placed); the word at `0xc0abbc` points at 16 random bytes less `0` (placed); the word at `p22+0` points at 12 random bytes less `0` (placed); the word at `p22+4` points at 12 random bytes less `0` (placed); the word at `p22+8` points at 12 random bytes less `0` (placed); the word at `p22+12` points at 12 random bytes less `0` (placed); the word at `0xc0abd8` points at 16 random bytes less `0` (placed); the word at `p27+0` points at 12 random bytes less `0` (placed); the word at `p27+4` points at 12 random bytes less `0` (placed); the word at `p27+8` points at 12 random bytes less `0` (placed); the word at `p27+12` points at 12 random bytes less `0` (placed); 4 rows 0x9d4 apart at `0xe85f20`, slices +0xc:0x4, +0x4a:0x4, +0x8cc:0x4 (`p` windows after the pointed buffers, before the places); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("army_member_by_unit") else {
        return;
    };
    let obj = super::Obj {
        name: "member",
        full: "Army::member@006f8de0",
        cells: 0,
        calls: false,
        eax: true,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, _log| {
        let _ = (&a, &m, &c, &p);
        // Whether unit `a[0]` of owner `a[1]` is a member of one of the army's groups: the army's
        // short count (`m+0x96`) of group ids from `m+0x54`, each a row k of the groups table
        // (`p[3k]` its count, `p[3k+1]` its owner byte at +0x4a, `p[3k+2]` its member shorts from
        // +0x8cc), counted when the owner is `a[1]` and the unit's object (the record `a[0]` of
        // owner `a[1]`'s table, `p[13 + 5 * who + a[0]]`, its flag byte at +8) is live (bit 0)
        // (item 2844).
        let count = i32::from(i16::from_le_bytes([m[0x96], m[0x97]]));
        let mut r = 0;
        'scan: for i in 0..count.max(0) {
            let gid = rd(m, 0x54 + 4 * i as usize);
            if gid < 0 {
                continue;
            }
            let k = gid as usize;
            if i32::from(p[3 * k + 1][0]) != a[1] {
                continue;
            }
            let who = a[1] as usize;
            if p[13 + 5 * who + a[0] as usize][8] & 1 == 0 {
                continue;
            }
            for j in 0..rd(&p[3 * k], 0).max(0) as usize {
                if i32::from(i16::from_le_bytes([
                    p[3 * k + 2][2 * j],
                    p[3 * k + 2][2 * j + 1],
                ])) == a[0]
                {
                    r = 1;
                    break 'scan;
                }
            }
        }
        r
    });
}
