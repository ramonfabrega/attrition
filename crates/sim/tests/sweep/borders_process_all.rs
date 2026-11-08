//! `Borders::process_all@00882490` under the emulator against the port (item 3044; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `Borders::process_all@00882490`, run by `tools/emu/sweep_obj.py` on its row `borders_process_all` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 16 random bytes of object, 0 argument(s), `p` windows in order: p0 the place; the global cell 0x00c0620c points at 576 random bytes; the global cell 0x00c0621c points at 576 random bytes; the global cell 0x00c06210 points at 672 random bytes; 4 random bytes at 0x00c061e0; callee 0x6d5f60 (2 args) stubbed; callee 0x0087fe30 (0 args) stubbed; the word at `c1+0x10` points at 16 random bytes less `0` (placed); stub 0 answered from [0, 1] (90%); stub 0 answers 8 values in call order, printed after the answers; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the recorded calls one token a call, in call order, each headed `sIDX:` (the callee); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 400 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("borders_process_all") else {
        return;
    };
    let obj = super::Obj {
        name: "process_all",
        full: "Borders::process_all@00882490",
        cells: 4,
        calls: true,
        eax: false,
        min_rows: 400,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, log| {
        let _ = (&a, &m, &c, &p);
        // `c0` the scene (`+0x22a` recalc flag), `c1` the borders (`+0x218` the visible mask, `+4` the
        // count, `+0x10` the list `p[0]`, `+0x161` a flag), `c2` the console (`+0x298` who), `c3` the
        // leaders cell. Stub 0 `can_see_borders(i, 0)` on the viewer's leader, answers `a[2..10]`
        // in call order; stub 1 `Border::process` on each listed border (the list's own words).
        if c[0][0x22a] != 0 || rd(&c[1], 0x218) == 0 {
            wr(&mut c[1], 0x218, 0);
            let who = rd(&c[2], 0x298);
            let ecx = (rd(&c[3], 0) as u32).wrapping_add((who as u32).wrapping_mul(0x6eec)) as i32;
            for i in 0..8 {
                log.push(format!("s0:{ecx},{i},0"));
                if a[2 + i] != 0 {
                    let mask = rd(&c[1], 0x218) | (1 << i);
                    wr(&mut c[1], 0x218, mask);
                }
            }
        }
        for i in 0..rd(&c[1], 4).max(0) as usize {
            log.push(format!("s1:{}", rd(&p[0], 4 * i)));
        }
        c[0][0x22a] = 0;
        c[1][0x161] = 0;
        if log.is_empty() {
            log.push("-".to_string());
        }
        0
    });
}
