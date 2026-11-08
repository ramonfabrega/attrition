//! `SpellType::cast_blow_tread@00672cb0` under the emulator against the port (item 2298; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `SpellType::cast_blow_tread@00672cb0`, run by `tools/emu/sweep_obj.py` on its row `spell_type_cast_blow_tread` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 32 random bytes of object, 4 argument(s), the global cell 0x00c06210 points at 672 random bytes; callee 0x0097f770 (1 args) stubbed; the word at `0xc0aec0+a[3]*0x1c` points at 4 random bytes less `a[2]*4` (placed); the word at `p0+0` points at 48 random bytes less `0` (placed); stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("spell_type_cast_blow_tread") else {
        return;
    };
    let obj = super::Obj {
        name: "cast_blow_tread",
        full: "SpellType::cast_blow_tread@00672cb0",
        cells: 1,
        calls: true,
        eax: false,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, log| {
        let _ = (&a, &m, &c, &p);
        {
            // The blow-tread sound (0x6c) when the console's player (+0x298) is
            // either side (a[1], a[3]); then unit a[2] of a[3]'s +0x24 set to 1
            // when it is 0.
            let who = rd(&c[0], 0x298);
            log.push(if a[1] == who || a[3] == who {
                "m+0x0,108".into()
            } else {
                "-".into()
            });
            if rd(&p[1], 0x24) == 0 {
                wr(&mut p[1], 0x24, 1);
            }
            0
        }
    });
}
