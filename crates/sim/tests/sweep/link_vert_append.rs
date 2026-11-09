//! `LinkVert::append@008821e0` under the emulator against the port (item 3038; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `LinkVert::append@008821e0`, run by `tools/emu/sweep_obj.py` on its row `link_vert_append` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 16 random bytes of object, 1 argument(s), `p` windows in order: p0 argument 0; 4 random bytes at 0x00ee1232; argument 0 names 8 random bytes; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 600 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("link_vert_append") else {
        return;
    };
    let obj = super::Obj {
        name: "append",
        full: "LinkVert::append@008821e0",
        cells: 1,
        calls: false,
        eax: true,
        min_rows: 600,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, _log| {
        let _ = (&a, &m, &c, &p);
        // A vertex links to at most two neighbours, `prev` (+4) then `next` (+0); the new one is
        // hung on whichever is empty and points back at `this` (the object's base, 0x20000000). With
        // both taken only the (muted) error report runs. The answer is the argument on the two linking arms, 0 on the third.
        const THIS: i32 = 0x2000_0000;
        if rd(m, 4) == 0 {
            wr(m, 4, a[0]);
            wr(&mut p[0], 0, THIS);
            i64::from(a[0])
        } else if rd(m, 0) == 0 {
            wr(m, 0, a[0]);
            wr(&mut p[0], 4, THIS);
            i64::from(a[0])
        } else {
            0
        }
    });
}
