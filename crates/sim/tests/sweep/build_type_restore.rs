//! `BuildType::restore@00631f80` under the emulator against the port (item 2271; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `BuildType::restore@00631f80`, run by `tools/emu/sweep_obj.py` on its row `build_type_restore` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 736 random bytes of object, 1 argument(s), argument 0 names 816 random bytes (null a third of the rows); callee 0x0065fb30 (1 args) stubbed; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("build_type_restore") else {
        return;
    };
    let obj = super::Obj {
        name: "restore",
        full: "BuildType::restore@00631f80",
        cells: 0,
        calls: true,
        eax: false,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, log| {
        let _ = (&a, &m, &c, &p);
        {
            // The tail `jmp` into the base's twin hands it `this` and
            // the record; a null record returns before it (item 2281).
            log.push(if a[0] != 0 { "m+0x0,p0+0x0" } else { "-" }.into());
            if a[0] != 0 {
                for (s, d) in [
                    (0x2b4, 0x314),
                    (0x2d4, 0x318),
                    (0x2d8, 0x31c),
                    (0x2c8, 0x320),
                    (0x2cc, 0x324),
                    (0x2d0, 0x32c),
                    (0x2c4, 0x328),
                ] {
                    let v = rd(&p[0], d);
                    wr(m, s, v);
                }
            }
            0
        }
    });
}
