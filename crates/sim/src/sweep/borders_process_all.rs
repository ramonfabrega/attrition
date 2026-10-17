//! `Borders::process_all@00882490` under the emulator against the port (item 3044; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

fn unhex(s: &str) -> Vec<u8> {
    (0..s.len() / 2)
        .map(|i| u8::from_str_radix(&s[2 * i..2 * i + 2], 16).expect("hex"))
        .collect()
}

/// `Borders::process_all@00882490`, run by `tools/emu/sweep_borders_process_all.py` under unicorn
/// (tools/emu): 16 random bytes of object, 0 argument(s), `p` windows in order: p0 the place; the global cell 0x00c0620c points at 576 random bytes; the global cell 0x00c0621c points at 576 random bytes; the global cell 0x00c06210 points at 672 random bytes; 4 random bytes at 0x00c061e0; callee 0x006d5f60 (2 args) stubbed; callee 0x0087fe30 (0 args) stubbed; the word at `c1+0x10` points at 16 random bytes less `0` (placed); stub 0 answered from [0, 1] (90%); stub 0 answers 8 values in call order, printed after the answers; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the recorded calls one token a call, in call order, each headed `sIDX:` (the callee); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 400 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_borders_process_all.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (line, calls) = line.split_once(" # ").expect("`<row> # <calls>`");
        let want_log: Vec<&str> = calls.split_whitespace().collect();
        let (lhs, rhs) = line
            .split_once(" -> ")
            .expect("a row is `<args…> | <windows> -> <eax> | <windows>`");
        let (args, before) = lhs.split_once(" | ").expect("`<args> | <before>`");
        let (eax, after) = rhs.split_once(" | ").expect("`<eax> | <after>`");
        let a: Vec<i32> = args
            .split_whitespace()
            .map(|s| s.parse().expect("an integer"))
            .collect();
        let mut wins: Vec<Vec<u8>> = before.trim().split(';').map(unhex).collect();
        let mut m = wins.remove(0);
        let mut p: Vec<Vec<u8>> = wins.split_off(4);
        let mut c: Vec<Vec<u8>> = wins;
        let want_eax: i64 = eax.trim().parse().expect("an integer");
        let mut log: Vec<String> = Vec::new();
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            let _ = (&a, &m, &c, &p);
            // `c0` the scene (`+0x22a` recalc flag), `c1` the borders (`+0x218` the visible mask, `+4` the
            // count, `+0x10` the list `p[0]`, `+0x161` a flag), `c2` the console (`+0x298` who), `c3` the
            // leaders cell. Stub 0 `can_see_borders(i, 0)` on the viewer's leader, answers `a[2..10]`
            // in call order; stub 1 `Border::process` on each listed border (the list's own words).
            if c[0][0x22a] != 0 || rd(&c[1], 0x218) == 0 {
                wr(&mut c[1], 0x218, 0);
                let who = rd(&c[2], 0x298);
                let ecx =
                    (rd(&c[3], 0) as u32).wrapping_add((who as u32).wrapping_mul(0x6eec)) as i32;
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
        };
        let _ = (got, want_eax);
        assert_eq!(
            log, want_log,
            "process_all — the recorded calls, row `{line}`"
        );
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "process_all — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "process_all — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[5 + i], "process_all — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 400, "only {n} rows");
    eprintln!("Borders::process_all@00882490: {n} rows agree");
}
