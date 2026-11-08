//! `ObjectData::inside_bottom@00644000` under the emulator against the port (item 2709; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `ObjectData::inside_bottom@00644000`, run by `tools/emu/sweep_obj.py` on its row `object_data_inside_bottom` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 12 random bytes of object, 1 argument(s), argument 0 names 4 random bytes (null a third of the rows); the word at `0xc0ab84+0x1c*0` points at 16 random bytes less `0` (placed); the word at `p1+0` points at 68 random bytes less `0` (placed); the word at `p1+4` points at 68 random bytes less `0` (placed); the word at `p1+8` points at 68 random bytes less `0` (placed); the word at `p1+12` points at 68 random bytes less `0` (placed); the word at `0xc0ab84+0x1c*1` points at 16 random bytes less `0` (placed); the word at `p6+0` points at 68 random bytes less `0` (placed); the word at `p6+4` points at 68 random bytes less `0` (placed); the word at `p6+8` points at 68 random bytes less `0` (placed); the word at `p6+12` points at 68 random bytes less `0` (placed); the word at `0xc0ab84+0x1c*2` points at 16 random bytes less `0` (placed); the word at `p11+0` points at 68 random bytes less `0` (placed); the word at `p11+4` points at 68 random bytes less `0` (placed); the word at `p11+8` points at 68 random bytes less `0` (placed); the word at `p11+12` points at 68 random bytes less `0` (placed); figures p2..p5 link at +0x28 (2 bytes) below their own index or to -1, -2; figures p7..p10 link at +0x28 (2 bytes) below their own index or to -1, -2; figures p12..p15 link at +0x28 (2 bytes) below their own index or to -1, -2; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("object_data_inside_bottom") else {
        return;
    };
    let obj = super::Obj {
        name: "inside_bottom",
        full: "ObjectData::inside_bottom@00644000",
        cells: 0,
        calls: false,
        eax: true,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, _log| {
        let _ = (&a, &m, &c, &p);
        let _ = &c;
        let r16 = |w: &[u8], o: usize| i32::from(i16::from_le_bytes([w[o], w[o + 1]]));
        let rec = |who: usize, idx: usize| 2 + 5 * who + idx;
        let mut who = usize::from(m[9]);
        let mut idx = usize::try_from(r16(m, 0xa)).expect("a column");
        let mut up = r16(&p[rec(who, idx)], 0x28);
        while up >= 0 {
            who = usize::from(p[rec(who, idx)][0x3e]);
            idx = usize::try_from(up).expect("a column");
            up = r16(&p[rec(who, idx)], 0x28);
        }
        if a[0] != 0 {
            wr(&mut p[0], 0, i32::try_from(who).expect("a row"));
        }
        i64::try_from(idx).expect("a column")
    });
}
