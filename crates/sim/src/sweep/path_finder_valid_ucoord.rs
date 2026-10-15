//! `PathFinder::valid_ucoord@00687c80` under the emulator against the port (item 2683; tools/sweep_gen.py, obj with windows).

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

/// `PathFinder::valid_ucoord@00687c80`, run by `tools/emu/sweep_path_finder_valid_ucoord.py` under unicorn
/// (tools/emu): 152 random bytes of object, 3 argument(s), the global cell 0x00c06188 points at 32 random bytes; the global cell 0x00cae5fc points at 128 random bytes; callee 0x00479b90 (1 args) stubbed; callee 0x00607c30 (8 args) stubbed; callee 0x00617060 (7 args) stubbed; callee 0x00479ef0 (2 args) stubbed; the word at `m+0x50` points at 16 random bytes less `0` (placed); the word at `m+0x54` points at 16 random bytes less `0` (placed); stub 0 answered from [0, 1] (90%); stub 1 answered from [0, 1] (90%); stub 2 answered from [0, 1] (90%); stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 3 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the recorded calls one token a call, in call order, each headed `sIDX:` (the callee); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_path_finder_valid_ucoord.py") else {
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
        let mut p: Vec<Vec<u8>> = wins.split_off(2);
        let mut c: Vec<Vec<u8>> = wins;
        let want_eax: i64 = eax.trim().parse().expect("an integer");
        let mut log: Vec<String> = Vec::new();
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            let _ = (&a, &m, &c, &p);
            {
                let (x, y, key) = (a[0], a[1], a[2]);
                let (w, h) = (rd(&c[0], 0x18), rd(&c[0], 0x1c));
                if x < 0 || y < 0 || x >= w.wrapping_mul(192) || y >= h.wrapping_mul(192) {
                    log.push("-".into());
                    0
                } else {
                    log.push(format!("s0:p0+0x0,{key}"));
                    if a[3] != 0 {
                        let n = rd(m, 0x90);
                        wr(m, 0x90, n.wrapping_add(1));
                        i64::from(rd(&p[0], 0) != 0)
                    } else {
                        let tx = rd(&c[1], 4 * (x >> 6) as usize);
                        let ty = rd(&c[1], 4 * (y >> 6) as usize);
                        log.push(format!("s1:p1+0x0,{tx},{ty},0,1,0,1,0,{}", y >> 6));
                        let mut r = 0;
                        if a[4] == 0 {
                            log.push(format!("s2:p1+0x0,{x},{y},1,1,p1+0x0,1,0"));
                            r = i64::from(a[5] == 0);
                        }
                        log.push(format!("s3:p0+0x0,{r},{key}"));
                        r
                    }
                }
            }
        };
        assert_eq!(got, want_eax, "valid_ucoord — eax, row `{line}`");
        assert_eq!(
            log, want_log,
            "valid_ucoord — the recorded calls, row `{line}`"
        );
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "valid_ucoord — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(
                w,
                &want[1 + i],
                "valid_ucoord — window c[{i}], row `{line}`"
            );
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(
                w,
                &want[3 + i],
                "valid_ucoord — window p[{i}], row `{line}`"
            );
        }
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("PathFinder::valid_ucoord@00687c80: {n} rows agree");
}
