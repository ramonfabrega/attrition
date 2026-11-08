//! `GroupData::group_air@0070b160` under the emulator against the port (item 2797; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `GroupData::group_air@0070b160`, run by `tools/emu/sweep_obj.py` on its row `group_data_group_air` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 2508 random bytes of object, 2 argument(s), argument 0 names 16 random bytes; vtable slot +0xc (4 args) stubbed; vtable slot +0x60 (2 args) stubbed; vtable slot +0xb8 (2 args) stubbed; the word at `0xc0ab84` points at 16 random bytes less `0` (placed); the word at `p1+0` points at 68 random bytes less `0` (placed); the word at `p1+4` points at 68 random bytes less `0` (placed); the word at `p1+8` points at 68 random bytes less `0` (placed); the word at `p1+12` points at 68 random bytes less `0` (placed); the word at `0xc0ab84+0x1c` points at 16 random bytes less `0` (placed); the word at `p6+0` points at 68 random bytes less `0` (placed); the word at `p6+4` points at 68 random bytes less `0` (placed); the word at `p6+8` points at 68 random bytes less `0` (placed); the word at `p6+12` points at 68 random bytes less `0` (placed); the word at `p2+0x18` points at 544 random bytes less `0` (placed); the word at `p3+0x18` points at 544 random bytes less `0` (placed); the word at `p4+0x18` points at 544 random bytes less `0` (placed); the word at `p5+0x18` points at 544 random bytes less `0` (placed); the word at `p7+0x18` points at 544 random bytes less `0` (placed); the word at `p8+0x18` points at 544 random bytes less `0` (placed); the word at `p9+0x18` points at 544 random bytes less `0` (placed); the word at `p10+0x18` points at 544 random bytes less `0` (placed); slot +0xb8 laid as 0x00653790 or its stub, a flag after the answers; stub 1 answered from [0, 1] (90%); stub 2 answered from [0, 1] (90%); stub 1 answers 4 values in call order, printed after the answers; stub 2 answers 4 values in call order, printed after the answers; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; figures p2..p5 link at +0x28 (2 bytes) above their own index or to -1; figures p7..p10 link at +0x28 (2 bytes) below their own index or to -1; the stubbed vtable laid at `p0+0` as well as at `this`; the stubbed vtable laid at `p11+0` as well as at `this`; the stubbed vtable laid at `p12+0` as well as at `this`; the stubbed vtable laid at `p13+0` as well as at `this`; the stubbed vtable laid at `p14+0` as well as at `this`; the stubbed vtable laid at `p15+0` as well as at `this`; the stubbed vtable laid at `p16+0` as well as at `this`; the stubbed vtable laid at `p17+0` as well as at `this`; the stubbed vtable laid at `p18+0` as well as at `this`; the stubbed vtable laid at `p2+0` as well as at `this`; the stubbed vtable laid at `p3+0` as well as at `this`; the stubbed vtable laid at `p4+0` as well as at `this`; the stubbed vtable laid at `p5+0` as well as at `this`; the stubbed vtable laid at `p7+0` as well as at `this`; the stubbed vtable laid at `p8+0` as well as at `this`; the stubbed vtable laid at `p9+0` as well as at `this`; the stubbed vtable laid at `p10+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("group_data_group_air") else {
        return;
    };
    let obj = super::Obj {
        name: "group_air",
        full: "GroupData::group_air@0070b160",
        cells: 0,
        calls: true,
        eax: false,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, log| {
        let _ = (&a, &m, &c, &p);
        {
            // Each listed object (`objects[who][list[k]]`) opens its passenger chain (+0x28 index, +0x3e table, -1
            // ends); a passenger whose type record (+0x18, its +0x218 word) is 2 reaches the group's slot +0xc with
            // `(index, table, 0, 0)` when `type_arg` is TYPE_NONE (-1) or `is(type_arg, 0)` answers non-zero: slot
            // +0xb8 is `ObjectData::is` (a[13], the type's +0x60, stub 1 a[5..9]) or its own stub (stub 2 a[9..13]).
            let num = rd(m, 12);
            let who = usize::from(m[0x4a]);
            let rec = |w: usize, i: usize| &p[2 + 5 * w + i];
            let ty = |w: usize, i: usize| &p[11 + 4 * w + i];
            let link = |o: &[u8]| (i16::from_le_bytes([o[0x28], o[0x29]]), o[0x3e] as i8);
            let (mut n_is, mut n_own) = (0usize, 0usize);
            let mut calls: Vec<String> = Vec::new();
            for k in 0..usize::try_from(num.max(0)).expect("a count") {
                let i = usize::try_from(i16::from_le_bytes([m[0x8cc + 2 * k], m[0x8cd + 2 * k]]))
                    .expect("an index");
                let (mut nx, mut nw) = link(rec(who, i));
                while nx >= 0 && nw >= 0 {
                    let (w2, i2) = (
                        usize::try_from(nw).expect("a table"),
                        usize::try_from(nx).expect("an index"),
                    );
                    if rd(ty(w2, i2), 0x218) == 2 {
                        let pass = a[1] == -1 || {
                            if a[13] != 0 {
                                n_is += 1;
                                a[5 + (n_is - 1).min(3)] != 0
                            } else {
                                n_own += 1;
                                a[9 + (n_own - 1).min(3)] != 0
                            }
                        };
                        if pass {
                            calls.push(format!("p0+0x0,{nx},{nw},0,0"));
                        }
                    }
                    (nx, nw) = link(rec(w2, i2));
                }
            }
            log.push(if calls.is_empty() {
                "-".into()
            } else {
                calls.join("/")
            });
            0
        }
    });
}
