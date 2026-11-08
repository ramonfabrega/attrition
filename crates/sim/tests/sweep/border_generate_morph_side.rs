//! `Border::generate_morph_side@00880050` under the emulator against the port (item 2818; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `Border::generate_morph_side@00880050`, run by `tools/emu/sweep_obj.py` on its row `border_generate_morph_side` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 8 random bytes of object, 1 argument(s), argument 0 names 16 random bytes; callee 0x004a1050 (0 args) stubbed; callee 0x00881b30 (2 args) stubbed; callee 0x004a1000 (0 args) stubbed; stub 0 answers the address of 32 random bytes, a `p` window after the places; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("border_generate_morph_side") else {
        return;
    };
    let obj = super::Obj {
        name: "generate_morph_side",
        full: "Border::generate_morph_side@00880050",
        cells: 0,
        calls: true,
        eax: true,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, log| {
        let _ = (&a, &m, &c, &p);
        {
            // A strip is popped (stub 0), takes the side's `who_bits` byte, and is filled by walk_strip (stub 1,
            // `ecx` the borders object + 0x164, handed the side and the strip's vertex list at +0xc); fewer than
            // two vertices and the strip is pushed back (stub 2) and the answer is null.
            p[1][9] = p[0][0xa];
            log.push("m+0x0".to_string());
            log.push("12659308,p0+0x0,p1+0xc".to_string());
            if rd(&p[1], 0x10) >= 2 {
                log.push("-".to_string());
                i64::from(a[1])
            } else {
                log.push("p1+0x0".to_string());
                0
            }
        }
    });
}
