//! `CityData::enhancer_level@007389d0` under the emulator against the port (item 2743; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `CityData::enhancer_level@007389d0`, run by `tools/emu/sweep_obj.py` on its row `city_data_enhancer_level` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 16 random bytes of object, 1 argument(s), callee 0x0066fcf0 (0 args) stubbed; callee 0x00739390 (3 args) stubbed; callee 0x00736890 (0 args) stubbed; callee 0x00736820 (0 args) stubbed; callee 0x007367a0 (0 args) stubbed; the word at `0xc096f4` points at 32 random bytes less `0` (placed); the word at `0xc061e4` points at 768 random bytes less `0` (placed); stub 1 answered from [0, 1] (90%); stub 0 answers -1..2 every row; stub 2 answers 0..3 every row; stub 3 answers 0..3 every row; stub 4 answers 0..3 every row; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("city_data_enhancer_level") else {
        return;
    };
    let obj = super::Obj {
        name: "enhancer_level",
        full: "CityData::enhancer_level@007389d0",
        cells: 0,
        calls: true,
        eax: true,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, log| {
        let _ = (&a, &m, &c, &p);
        // `CityData::enhancer_level(kind)`: kinds 0 and 1 need the city's own flag (+4: 0x200, 0x400); the
        // rest ask the type at `0xc096f4[kind]` (`0x66fcf0`, a negative answer ends it) and the city
        // (`0x739390`, zero ends it, as does a kind past 5). The surviving kinds 0, 1 and 4 read a level
        // from the table at `0xc061e4` — `+0x2b8`, `+0x2cc`, `+0x2e0` — by the index a city callee answers.
        let (kind, s66, s39) = (a[0], a[1], a[2]);
        let (mut t0, mut t1) = ("-".to_string(), "-".to_string());
        let open = match kind {
            0 => m[5] & 2 != 0,
            1 => m[5] & 4 != 0,
            _ => {
                t0 = rd(&p[0], 4 * kind as usize).to_string();
                if s66 < 0 {
                    false
                } else {
                    t1 = format!("m+0x0,{s66},0,1");
                    s39 != 0 && kind <= 5
                }
            }
        };
        let level = |base: usize, idx: i32| rd(&p[1], base + 4 * idx as usize);
        let r = if !open {
            0
        } else {
            match kind {
                0 => level(0x2b8, a[3]),
                1 => level(0x2cc, a[4]),
                4 => level(0x2e0, a[5]),
                _ => 0,
            }
        };
        log.push(t0);
        log.push(t1);
        i64::from(r)
    });
}
