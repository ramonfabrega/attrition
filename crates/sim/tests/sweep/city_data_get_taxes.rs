//! `CityData::get_taxes@00737b50` under the emulator against the port (item 2281; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `CityData::get_taxes@00737b50`, run by `tools/emu/sweep_obj.py` on its row `city_data_get_taxes` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 96 random bytes of object, 0 argument(s), the global cell 0x00c061e4 points at 1208 random bytes; 4 random bytes at 0x00c061e0; callee 0x00738190 (0 args) stubbed; callee 0x00739390 (3 args) stubbed; callee 0x006ebc10 (1 args) stubbed; stub 0 answered from [0, 1, 2, 5, 30] (90%); stub 1 answered from [0, 1, 2] (90%); stub 2 answered from [0, 1] (90%); stub 1 answers 2 values in call order, printed after the answers; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("city_data_get_taxes") else {
        return;
    };
    let obj = super::Obj {
        name: "get_taxes",
        full: "CityData::get_taxes@00737b50",
        cells: 2,
        calls: true,
        eax: true,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, log| {
        let _ = (&a, &m, &c, &p);
        {
            // A village's base and its buildings; the market (0x1b4) and, with the
            // owner's porcelain wonder (0x215), its percentage; the temple (0x1b5).
            let k = &c[0];
            let mut t = rd(k, 0x324).wrapping_add(a[0].wrapping_mul(rd(k, 0x328)));
            log.push("m+0x0".into());
            log.push("m+0x0,436,0,1/m+0x0,437,0,1".into());
            if a[3] != 0 {
                let mut v = rd(k, 0x32c);
                let leader =
                    rd(&c[1], 0).wrapping_add(i32::from(m[0x5e] as i8).wrapping_mul(0x6eec));
                log.push(format!("{leader},533"));
                if a[2] != 0 {
                    v = rd(k, 0x4b4).wrapping_add(100).wrapping_mul(v) / 100;
                }
                t = t.wrapping_add(v);
            } else {
                log.push("-".into());
            }
            if a[4] != 0 {
                t = t.wrapping_add(rd(k, 0x330));
            }
            i64::from(t)
        }
    });
}
