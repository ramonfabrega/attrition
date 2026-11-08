//! `CityData::num_kinds@007366c0` under the emulator against the port (item 2276; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `CityData::num_kinds@007366c0`, run by `tools/emu/sweep_obj.py` on its row `city_data_num_kinds` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 96 random bytes of object, 0 argument(s), the word at `0xc0ab84+2*0x1c` points at 24 random bytes less `0` (placed); the word at `p0+0` points at 120 random bytes less `0` (placed); the word at `p0+4` points at 120 random bytes less `0` (placed); the word at `p0+8` points at 120 random bytes less `0` (placed); the word at `p0+12` points at 120 random bytes less `0` (placed); the word at `p0+16` points at 120 random bytes less `0` (placed); the word at `p0+20` points at 120 random bytes less `0` (placed); the word at `p1+0x18` points at 8 random bytes less `0` (placed); the word at `p2+0x18` points at 8 random bytes less `0` (placed); the word at `p3+0x18` points at 8 random bytes less `0` (placed); the word at `p4+0x18` points at 8 random bytes less `0` (placed); the word at `p5+0x18` points at 8 random bytes less `0` (placed); the word at `p6+0x18` points at 8 random bytes less `0` (placed); figures p1..p6 link at +0x74 (2 bytes) above their own index or to -1; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("city_data_num_kinds") else {
        return;
    };
    let obj = super::Obj {
        name: "num_kinds",
        full: "CityData::num_kinds@007366c0",
        cells: 0,
        calls: false,
        eax: true,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, _log| {
        let _ = (&a, &m, &c, &p);
        {
            // The city's chain head `+8`, its player `+0x5e` (held at 2),
            // the objects table `p[0]` of six `Build`s `p[1..7]` (vtable
            // `0xb42174`), each's next `+0x74`, `flags` `+8`, its type
            // record `p[7 + k]` (`+0x18`) whose `+4` is the kind.
            let mut chain = Vec::new();
            let mut at = i16::from_le_bytes([m[8], m[9]]);
            while at >= 0 {
                chain.push(at as usize);
                at = i16::from_le_bytes([p[1 + at as usize][0x74], p[1 + at as usize][0x75]]);
            }
            let (n, _) = crate::city::distinct_kinds(
                chain,
                |b| p[1 + b][8] & 5 == 5,
                |b| Some(rd(&p[7 + b], 4)),
                i32::MAX,
            );
            i64::from(n)
        }
    });
}
