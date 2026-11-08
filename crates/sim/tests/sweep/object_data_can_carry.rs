//! `ObjectData::can_carry@00646c40` under the emulator against the port (item 2852; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `ObjectData::can_carry@00646c40`, run by `tools/emu/sweep_obj.py` on its row `object_data_can_carry` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 32 random bytes of object, 1 argument(s), vtable slot +0x18 (0 args) stubbed; vtable slot +0xb8 (2 args) stubbed; vtable slot +0x60 (2 args) stubbed; the word at `m+0x18` points at 8 random bytes less `0` (placed); the word at `0xc0aec0` points at 8 random bytes less `0` (placed); the word at `0xc0aedc` points at 8 random bytes less `0` (placed); the word at `p1+0` points at 32 random bytes less `0` (placed); the word at `p1+4` points at 32 random bytes less `0` (placed); the word at `p2+0` points at 32 random bytes less `0` (placed); the word at `p2+4` points at 32 random bytes less `0` (placed); the word at `p3+0x18` points at 728 random bytes less `0` (placed); the word at `p4+0x18` points at 728 random bytes less `0` (placed); the word at `p5+0x18` points at 728 random bytes less `0` (placed); the word at `p6+0x18` points at 728 random bytes less `0` (placed); slot +0xb8 laid as 0x00653790 or its stub, a flag after the answers; stub 0 answered from [0, 1, 1] (90%); stub 1 answered from [0, 1] (90%); stub 2 answered from [0, 1] (90%); stub 1 answers 3 values in call order, printed after the answers; stub 2 answers 3 values in call order, printed after the answers; the stubbed vtable laid at `p0+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("object_data_can_carry") else {
        return;
    };
    let obj = super::Obj {
        name: "can_carry",
        full: "ObjectData::can_carry@00646c40",
        cells: 0,
        calls: false,
        eax: true,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, _log| {
        let _ = (&a, &m, &c, &p);
        // `a[0]` is the domain: 2 (air) is carried when any of three asks answers yes in turn, 0 (ground) when slot
        // +0x18 (`a[1]`) is set, the carried type's record (`p[7 + 2·who + idx]`, `who` the byte at `m+9`, `idx` the
        // short at `m+0xa`) has a word at +0x2d4, and the one ask answers no; any other domain is not carried. An
        // ask is slot +0xb8's answer (`a[4..7]` in call order) or, when it is laid as the base (flag `a[10]`),
        // the object at `m+0x18`'s slot +0x60's (`a[7..10]`) (item 2852).
        let (mut nb, mut n6) = (0, 0);
        let mut ask = || {
            if a[10] != 0 {
                n6 += 1;
                a[6 + n6] != 0
            } else {
                nb += 1;
                a[3 + nb] != 0
            }
        };
        let carried = match a[0] {
            2 => ask() || ask() || ask(),
            0 => {
                let rec = &p[7
                    + 2 * usize::from(m[9])
                    + usize::from(i16::from_le_bytes([m[10], m[11]]) as u16)];
                a[1] != 0 && rd(rec, 0x2d4) != 0 && !ask()
            }
            _ => false,
        };
        i64::from(carried)
    });
}
