//! `Build::repair_damage@00628130` under the emulator against the port (item 2211; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `Build::repair_damage@00628130`, run by `tools/emu/sweep_obj.py` on its row `build_repair_damage` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 60 random bytes of object, 3 argument(s), vtable slot +0x11c (1 args) stubbed; stub 0 answered from [-3, 0, 1, 5, 10, 20] (90%); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("build_repair_damage") else {
        return;
    };
    let obj = super::Obj {
        name: "repair_damage",
        full: "Build::repair_damage@00628130",
        cells: 0,
        calls: false,
        eax: false,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, _log| {
        let _ = (&a, &m, &c, &p);
        {
            let mut h = rd(m, 0x24);
            if h > a[3] {
                h = a[3];
            }
            m[0x3b] = 0;
            if a[0] >= h {
                wr(m, 0x24, 0);
            } else {
                wr(m, 0x24, h.wrapping_sub(a[0]));
            }
            0
        }
    });
}
