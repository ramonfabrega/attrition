//! `ObjectsData::find_city@0065ba90` under the emulator against the port (item 3030; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `ObjectsData::find_city@0065ba90`, run by `tools/emu/sweep_obj.py` on its row `objects_data_find_city` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 32 random bytes of object, 9 argument(s), `p` windows in order: p0..p15 sparse table 0 (8 rows × 2 slice(s), row-major), p16..p88 the place(s), in flag order; the global cell 0x00c0618c points at 528 random bytes; the global cell 0x00cae5fc points at 512 random bytes; 320 random bytes at 0x00c097e8; callee 0x0067daa0 (2 args) stubbed; callee 0x0067dbb0 (4 args) stubbed; the word at `0xc09970` points at 12 random bytes less `0` (placed); the word at `p16+0` points at 16 random bytes less `0` (placed); the word at `p16+4` points at 16 random bytes less `0` (placed); the word at `p16+8` points at 16 random bytes less `0` (placed); the word at `0xc0998c` points at 12 random bytes less `0` (placed); the word at `p20+0` points at 16 random bytes less `0` (placed); the word at `p20+4` points at 16 random bytes less `0` (placed); the word at `p20+8` points at 16 random bytes less `0` (placed); the word at `0xc099a8` points at 12 random bytes less `0` (placed); the word at `p24+0` points at 16 random bytes less `0` (placed); the word at `p24+4` points at 16 random bytes less `0` (placed); the word at `p24+8` points at 16 random bytes less `0` (placed); the word at `0xc099c4` points at 12 random bytes less `0` (placed); the word at `p28+0` points at 16 random bytes less `0` (placed); the word at `p28+4` points at 16 random bytes less `0` (placed); the word at `p28+8` points at 16 random bytes less `0` (placed); the word at `0xc099e0` points at 12 random bytes less `0` (placed); the word at `p32+0` points at 16 random bytes less `0` (placed); the word at `p32+4` points at 16 random bytes less `0` (placed); the word at `p32+8` points at 16 random bytes less `0` (placed); the word at `0xc099fc` points at 12 random bytes less `0` (placed); the word at `p36+0` points at 16 random bytes less `0` (placed); the word at `p36+4` points at 16 random bytes less `0` (placed); the word at `p36+8` points at 16 random bytes less `0` (placed); the word at `0xc09a18` points at 12 random bytes less `0` (placed); the word at `p40+0` points at 16 random bytes less `0` (placed); the word at `p40+4` points at 16 random bytes less `0` (placed); the word at `p40+8` points at 16 random bytes less `0` (placed); the word at `0xc09a34` points at 12 random bytes less `0` (placed); the word at `p44+0` points at 16 random bytes less `0` (placed); the word at `p44+4` points at 16 random bytes less `0` (placed); the word at `p44+8` points at 16 random bytes less `0` (placed); the word at `0xc0ab84` points at 16 random bytes less `0` (placed); the word at `p48+0` points at 24 random bytes less `0` (placed); the word at `p48+4` points at 24 random bytes less `0` (placed); the word at `p48+8` points at 24 random bytes less `0` (placed); the word at `p48+12` points at 24 random bytes less `0` (placed); the word at `0xc0aba0` points at 16 random bytes less `0` (placed); the word at `p53+0` points at 24 random bytes less `0` (placed); the word at `p53+4` points at 24 random bytes less `0` (placed); the word at `p53+8` points at 24 random bytes less `0` (placed); the word at `p53+12` points at 24 random bytes less `0` (placed); the word at `0xc0abbc` points at 16 random bytes less `0` (placed); the word at `p58+0` points at 24 random bytes less `0` (placed); the word at `p58+4` points at 24 random bytes less `0` (placed); the word at `p58+8` points at 24 random bytes less `0` (placed); the word at `p58+12` points at 24 random bytes less `0` (placed); the word at `0xc0abd8` points at 16 random bytes less `0` (placed); the word at `p63+0` points at 24 random bytes less `0` (placed); the word at `p63+4` points at 24 random bytes less `0` (placed); the word at `p63+8` points at 24 random bytes less `0` (placed); the word at `p63+12` points at 24 random bytes less `0` (placed); the word at `0xc0abf4` points at 16 random bytes less `0` (placed); the word at `p68+0` points at 24 random bytes less `0` (placed); the word at `p68+4` points at 24 random bytes less `0` (placed); the word at `p68+8` points at 24 random bytes less `0` (placed); the word at `p68+12` points at 24 random bytes less `0` (placed); the word at `0xc0ac10` points at 16 random bytes less `0` (placed); the word at `p73+0` points at 24 random bytes less `0` (placed); the word at `p73+4` points at 24 random bytes less `0` (placed); the word at `p73+8` points at 24 random bytes less `0` (placed); the word at `p73+12` points at 24 random bytes less `0` (placed); the word at `0xc0ac2c` points at 16 random bytes less `0` (placed); the word at `p78+0` points at 24 random bytes less `0` (placed); the word at `p78+4` points at 24 random bytes less `0` (placed); the word at `p78+8` points at 24 random bytes less `0` (placed); the word at `p78+12` points at 24 random bytes less `0` (placed); the word at `0xc0ac48` points at 16 random bytes less `0` (placed); the word at `p83+0` points at 24 random bytes less `0` (placed); the word at `p83+4` points at 24 random bytes less `0` (placed); the word at `p83+8` points at 24 random bytes less `0` (placed); the word at `p83+12` points at 24 random bytes less `0` (placed); the word at `c2+0x134` points at 2816 random bytes less `0` (placed); stub 0 answers 8 values in call order, printed after the answers; stub 1 answers 24 values in call order, printed after the answers; stub 0 answers 0..1 every row; stub 1 answers 0..1 every row; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; 8 rows 0x6eec apart at `@0xe3a390`, slices +0x0:0x4, +0x408:0x4 (`p` windows after the pointed buffers, before the places); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 800 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("objects_data_find_city") else {
        return;
    };
    let obj = super::Obj {
        name: "find_city",
        full: "ObjectsData::find_city@0065ba90",
        cells: 3,
        calls: true,
        eax: true,
        min_rows: 800,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, log| {
        let _ = (&a, &m, &c, &p);
        let d3 = |i: i32| rd(&c[1], 4 * i as usize);
        let (x, y, srch, p4, p6, p7, p8, p9) = (a[0], a[1], a[2], a[3], a[5], a[6], a[7], a[8]);
        let sh = |w: &[u8], o: usize| i32::from(i16::from_le_bytes([w[o], w[o + 1]]));
        let mut owner = -1;
        if p6 & 0x200 != 0 {
            let w = rd(&c[2], 0);
            let idx = (d3(y >> 8) * w + d3(x >> 8)) as usize;
            owner = sh(&p[88], 0x1c * idx + 4);
        }
        wr(&mut c[0], 0x200, p4);
        wr(&mut c[0], 0x1fc, 99_999_999);
        let (mut k_s, mut k_f) = (0usize, 0usize);
        let (mut ls, mut lf): (Vec<String>, Vec<String>) = (Vec::new(), Vec::new());
        let mut best = -1i64;
        for k in 0..8usize {
            if rd(&p[2 * k], 0) & 1 == 0 {
                continue;
            }
            let ok = a[11 + k_s];
            k_s += 1;
            ls.push(format!("{k},{srch},{p4}"));
            if ok == 0 {
                continue;
            }
            for i in 0..rd(&p[2 * k + 1], 0).max(0) as usize {
                let city = &p[17 + 4 * k + i];
                if rd(city, 4) & 1 == 0 {
                    continue;
                }
                let cs = sh(city, 8);
                if p6 & 0x200 != 0 && owner != sh(city, 10) {
                    continue;
                }
                let unit = &p[49 + 5 * k + cs as usize];
                let dy = (rd(unit, 0x14) ^ 0x63637).wrapping_sub(y).wrapping_abs();
                let dx = (rd(unit, 0x10) ^ 0x63637).wrapping_sub(x).wrapping_abs();
                let d = crate::world::vector_dist(dx, dy);
                if d > rd(&c[0], 0x1fc) {
                    continue;
                }
                let f = p7.wrapping_sub(1) as u32;
                if p7 != 0 && f <= 0x16 {
                    let ok = a[19 + k_f];
                    k_f += 1;
                    lf.push(format!("{k},{cs},{p8},{p9},{f}"));
                    if ok == 0 {
                        continue;
                    }
                }
                wr(&mut c[0], 0x1fc, d);
                wr(&mut c[0], 0x200, k as i32);
                best = i as i64;
            }
        }
        log.push(if ls.is_empty() {
            "-".into()
        } else {
            ls.join("/")
        });
        log.push(if lf.is_empty() {
            "-".into()
        } else {
            lf.join("/")
        });
        best
    });
}
