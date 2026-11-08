//! `Unit::update_ceo_position@005f96b0` under the emulator against the port (item 3044; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `Unit::update_ceo_position@005f96b0`, run by `tools/emu/sweep_obj.py` on its row `unit_update_ceo_position` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 16 random bytes of object, 4 argument(s), `p` windows in order: p0..p7 sparse table 0 (8 rows × 1 slice(s), row-major); the global cell 0x00c06188 points at 32 random bytes; 4 random bytes at 0x00c0618c; callee 0x0065b870 (5 args) stubbed; stub 0 answered from [-1, 0, 1] (90%); stub 0 answers 2 values in call order, printed after the answers; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the recorded calls one token a call, in call order, each headed `sIDX:` (the callee); 8 rows 0x6eec apart at `@0xe3a390`, slices +0x0:0x4 (`p` windows after the pointed buffers, before the places); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 400 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("unit_update_ceo_position") else {
        return;
    };
    let obj = super::Obj {
        name: "update_ceo_position",
        full: "Unit::update_ceo_position@005f96b0",
        cells: 2,
        calls: true,
        eax: false,
        min_rows: 400,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, log| {
        let _ = (&a, &m, &c, &p);
        // `a[0..2]` and `a[2..4]` two tiles, each looked up for a city (`find_city_at`, stub 0, its
        // answers `a[5..7]` in call order) if inside `c0`'s world (`+0x18` wide, `+0x1c` tall); the
        // owner `m[9]`'s leader flags (`p[owner]`) take 0x2000000 when the two lookups differ.
        let (w, h) = (rd(&c[0], 0x18), rd(&c[0], 0x1c));
        let owner = usize::from(m[9]);
        let ecx = rd(&c[1], 0);
        let mut turn = 0;
        let mut look = |x: i32, y: i32, log: &mut Vec<String>| -> i32 {
            if x >= 0 && y >= 0 && x < w && y < h {
                log.push(format!(
                    "s0:{ecx},{},{},{owner},-1,0",
                    x * 0xc0 + 0x60,
                    y * 0xc0 + 0x60
                ));
                turn += 1;
                a[4 + turn]
            } else {
                -1
            }
        };
        let first = look(a[0], a[1], &mut *log);
        let second = look(a[2], a[3], &mut *log);
        if first != second {
            let flags = rd(&p[owner], 0) | 0x0200_0000;
            wr(&mut p[owner], 0, flags);
        }
        if log.is_empty() {
            log.push("-".to_string());
        }
        0
    });
}
