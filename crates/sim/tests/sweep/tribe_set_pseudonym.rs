//! `Tribe::set_pseudonym@006f0200` under the emulator against the port (item 2495; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `Tribe::set_pseudonym@006f0200`, run by `tools/emu/sweep_obj.py` on its row `tribe_set_pseudonym` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 96 random bytes of object, 1 argument(s), callee 0x00a1eeb0 (1 args) stubbed; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("tribe_set_pseudonym") else {
        return;
    };
    let obj = super::Obj {
        name: "set_pseudonym",
        full: "Tribe::set_pseudonym@006f0200",
        cells: 0,
        calls: true,
        eax: false,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, log| {
        let _ = (&a, &m, &c, &p);
        {
            if rd(m, 0x48) & 0xffff == 0 {
                log.push(format!("m+0x40,m+0x18/m+0x18,{}", a[0]));
            } else {
                log.push(format!("m+0x18,{}", a[0]));
            }
            0
        }
    });
}
