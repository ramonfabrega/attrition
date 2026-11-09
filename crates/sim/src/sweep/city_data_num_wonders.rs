//! `CityData::num_wonders@007382b0` under the emulator against the port (item 2276; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `CityData::num_wonders@007382b0`, run by `tools/emu/sweep_obj.py` on its row `city_data_num_wonders` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 96 random bytes of object, 1 argument(s), vtable slot +0x1c (0 args) stubbed; vtable slot +0x60 (2 args) stubbed; the word at `0xc0ab84+2*0x1c` points at 24 random bytes less `0` (placed); the word at `p0+0` points at 120 random bytes less `0` (placed); the word at `p0+4` points at 120 random bytes less `0` (placed); the word at `p0+8` points at 120 random bytes less `0` (placed); the word at `p0+12` points at 120 random bytes less `0` (placed); the word at `p0+16` points at 120 random bytes less `0` (placed); the word at `p0+20` points at 120 random bytes less `0` (placed); the word at `p1+0x18` points at 8 random bytes less `0` (placed); the word at `p2+0x18` points at 8 random bytes less `0` (placed); the word at `p3+0x18` points at 8 random bytes less `0` (placed); the word at `p4+0x18` points at 8 random bytes less `0` (placed); the word at `p5+0x18` points at 8 random bytes less `0` (placed); the word at `p6+0x18` points at 8 random bytes less `0` (placed); stub 0 answered from [0, 1] (90%); stub 1 answered from [0, 0, 1] (90%); stub 0 answers 6 values in call order, printed after the answers; stub 1 answers 6 values in call order, printed after the answers; figures p1..p6 link at +0x74 (2 bytes) above their own index or to -1; the stubbed vtable laid at `p7+0` as well as at `this`; the stubbed vtable laid at `p8+0` as well as at `this`; the stubbed vtable laid at `p9+0` as well as at `this`; the stubbed vtable laid at `p10+0` as well as at `this`; the stubbed vtable laid at `p11+0` as well as at `this`; the stubbed vtable laid at `p12+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("city_data_num_wonders") else {
        return;
    };
    let obj = super::Obj {
        name: "num_wonders",
        full: "CityData::num_wonders@007382b0",
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
            // record `p[7 + k]` carrying the stubbed vtable: `is_wonder`
            // (`+0x1c`) answers `a[3..9]`, `is(0x216)` (`+0x60`)
            // `a[9..15]`, in call order. `a[0]` is `exclude_city`.
            let mut chain = Vec::new();
            let mut at = i16::from_le_bytes([m[8], m[9]]);
            while at >= 0 {
                chain.push(at as usize);
                at = i16::from_le_bytes([p[1 + at as usize][0x74], p[1 + at as usize][0x75]]);
            }
            let (mut w, mut r) = (0usize, 0usize);
            i64::from(crate::cities::city::count_wonders(
                chain,
                |_| {
                    w += 1;
                    a[3 + (w - 1).min(5)] != 0
                },
                |_| {
                    r += 1;
                    a[9 + (r - 1).min(5)] != 0
                },
                |b| a[0] != 0 && p[1 + b][8] & 0x20 != 0,
            ))
        }
    });
}
