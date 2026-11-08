//! `ObjectsData::find_goody_at@0065b7c0` under the emulator against the port (item 2743; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `ObjectsData::find_goody_at@0065b7c0`, run by `tools/emu/sweep_obj.py` on its row `objects_data_find_goody_at` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 16 random bytes of object, 3 argument(s), callee 0x0065c040 (1 args) stubbed; the word at `0xcae5fc` points at 64 random bytes less `0` (placed); the word at `0xc061d0` points at 320 random bytes less `0` (placed); the word at `p1+0x134` points at 448 random bytes less `0` (placed); stub 0 answered from [0, 1, 5, -7] (90%); stub 0 prints the 4 bytes its argument 0 points at, hex, after the record (`/` a call); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("objects_data_find_goody_at") else {
        return;
    };
    let obj = super::Obj {
        name: "find_goody_at",
        full: "ObjectsData::find_goody_at@0065b7c0",
        cells: 0,
        calls: true,
        eax: true,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, log| {
        let _ = (&a, &m, &c, &p);
        // Two handles (`>> 8`, through the table at `0xcae5fc`) name a tile (x, y) of the map at `0xc061d0`
        // (width at +0, 28-byte tiles at the pointer in +0x134): a tile without the 0x100 flag whose byte
        // +2 is 1 or 2 answers -1; any other goes to the overload at `0x65c040`, handed the y handle.
        let hx = rd(&p[0], 4 * (a[0] >> 8) as usize);
        let hy = rd(&p[0], 4 * (a[1] >> 8) as usize);
        let at = (rd(&p[1], 0).wrapping_mul(hy).wrapping_add(hx) as usize) * 28;
        let tile = &p[2][at..at + 28];
        let flagged = u16::from_le_bytes([tile[0], tile[1]]) & 0x100 != 0;
        if !flagged && matches!(tile[2] as i8, 1 | 2) {
            log.push("-".into());
            -1
        } else {
            log.push(
                hy.to_le_bytes()
                    .iter()
                    .map(|b| format!("{b:02x}"))
                    .collect(),
            );
            i64::from(a[3])
        }
    });
}
