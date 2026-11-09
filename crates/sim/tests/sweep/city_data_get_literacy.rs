//! `CityData::get_literacy@00737c00` under the emulator against the port (item 2336; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `CityData::get_literacy@00737c00`, run by `tools/emu/sweep_obj.py` on its row `city_data_get_literacy` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 64 random bytes of object, 0 argument(s), the global cell 0x00c061e4 points at 852 random bytes; callee 0x00739390 (3 args) stubbed; stub 0 answered from [0, 1, 2, -1] (90%); stub 0 answers 2 values in call order, printed after the answers; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("city_data_get_literacy") else {
        return;
    };
    let obj = super::Obj {
        name: "get_literacy",
        full: "CityData::get_literacy@00737c00",
        cells: 1,
        calls: true,
        eax: true,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, log| {
        let _ = (&a, &m, &c, &p);
        {
            // The village literacy (+0x348 of the constants, cell 0xc061e4), the university's
            // (+0x34c) added when `count_buildings` (stub 0) finds a university (0x1a4), the
            // library's (+0x350) when it finds a library (0x1b3); each count `(type, 0, 1)`.
            log.push("m+0x0,420,0,1/m+0x0,435,0,1".into());
            let mut v = rd(&c[0], 0x348);
            if a[1] != 0 {
                v = v.wrapping_add(rd(&c[0], 0x34c));
            }
            if a[2] != 0 {
                v = v.wrapping_add(rd(&c[0], 0x350));
            }
            i64::from(v)
        }
    });
}
