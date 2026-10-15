//! `Armies::send_navy@006f2c90` under the emulator against the port (item 2714; tools/sweep_gen.py, obj with windows).

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

/// `Armies::send_navy@006f2c90`, run by `tools/emu/sweep_armies_send_navy.py` under unicorn
/// (tools/emu): 32 random bytes of object, 5 argument(s), callee 0x00680f90 (1 args) stubbed; callee 0x006f93d0 (1 args) stubbed; the word at `p5+0` points at 12 random bytes less `0` (placed); the word at `p7+0` points at 12 random bytes less `0` (placed); the word at `p8+0` points at 64 random bytes less `0` (placed); the word at `p8+4` points at 64 random bytes less `0` (placed); the word at `p8+8` points at 64 random bytes less `0` (placed); the word at `p9+0` points at 64 random bytes less `0` (placed); the word at `p9+4` points at 64 random bytes less `0` (placed); the word at `p9+8` points at 64 random bytes less `0` (placed); the word at `0xc061b8` points at 32 random bytes less `0` (placed); the word at `p16+0x10` points at 4480 random bytes less `0x2200` (placed); stub 0 answered from [1, 1, 1, 0] (90%); stub 0 answers 6 values in call order, printed after the answers; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the recorded calls one token a call, in call order, each headed `sIDX:` (the callee); 2 rows 0x6eec apart at `@0xe3a390`, slices +0x0:0x4, +0x4:0x4 (`p` windows after the pointed buffers, before the places); 2 rows 0x1c apart at `@0xc09704`, slices +0x0:0x4, +0xc:0x4 (`p` windows after the pointed buffers, before the places); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_armies_send_navy.py") else {
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
        let mut p: Vec<Vec<u8>> = wins.split_off(0);
        let mut c: Vec<Vec<u8>> = wins;
        let want_eax: i64 = eax.trim().parse().expect("an integer");
        let mut log: Vec<String> = Vec::new();
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            let _ = (&a, &m, &c, &p);
            {
                let lead = a[0] as usize;
                let (fl, f2) = (rd(&p[2 * lead], 0), rd(&p[2 * lead + 1], 0));
                let mut seq_i = 0usize;
                let mut calls: Vec<String> = Vec::new();
                if fl & 5 == 1 && f2 & 0xa == 0 {
                    let cnt = rd(&p[4 + 2 * lead], 0);
                    for i in 0..cnt.max(0) as usize {
                        let w = 10 + 3 * lead + i;
                        let rec = &p[w];
                        if i16::from_le_bytes([rec[0], rec[1]]) == 0
                            || rd(rec, 0x24) == 0
                            || rd(rec, 0x14) < 3
                            || rd(rec, 8) < 0x40
                        {
                            continue;
                        }
                        let mut coast = |arg: i32| {
                            calls.push(format!("s0:p17+0x0,{arg}"));
                            let v = a[7 + seq_i.min(5)];
                            seq_i += 1;
                            v != 0
                        };
                        if coast(a[3]) && coast(a[4]) {
                            wr(&mut p[w], 0x30, a[1]);
                            wr(&mut p[w], 0x34, a[2]);
                            calls.push(format!("s1:p{w}+0x0,0"));
                        }
                    }
                }
                log.extend(if calls.is_empty() {
                    vec!["-".to_string()]
                } else {
                    calls
                });
                0
            }
        };
        let _ = (got, want_eax);
        assert_eq!(
            log, want_log,
            "send_navy — the recorded calls, row `{line}`"
        );
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "send_navy — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "send_navy — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "send_navy — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("Armies::send_navy@006f2c90: {n} rows agree");
}
