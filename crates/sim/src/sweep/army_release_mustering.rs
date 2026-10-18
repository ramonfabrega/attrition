//! `Army::release_mustering@006f87c0` under the emulator against the port (item 3200; tools/sweep_gen.py, obj with windows).

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

/// `Army::release_mustering@006f87c0`, run by `tools/emu/sweep_army_release_mustering.py` under unicorn
/// (tools/emu): 256 random bytes of object, 0 argument(s), `p` windows in order: p0..p55 sparse table 0 (8 rows × 7 slice(s), row-major), p56..p82 the place(s), in flag order; the global cell 0x00c061ec points at 64 random bytes; callee 0x00594670 (0 args) stubbed; callee 0x006edb50 (1 args) stubbed; callee 0x006ec000 (0 args) stubbed; callee 0x006f3200 (3 args) stubbed; the word at `12622192` points at 64 random bytes less `0` (placed); the word at `12622220` points at 64 random bytes less `0` (placed); the word at `12622248` points at 64 random bytes less `0` (placed); the word at `12622276` points at 64 random bytes less `0` (placed); the word at `12622304` points at 64 random bytes less `0` (placed); the word at `12622332` points at 64 random bytes less `0` (placed); the word at `12622360` points at 64 random bytes less `0` (placed); the word at `12622388` points at 64 random bytes less `0` (placed); the word at `p56+0` points at 128 random bytes less `0` (placed); the word at `p56+4` points at 128 random bytes less `0` (placed); the word at `p57+0` points at 128 random bytes less `0` (placed); the word at `p57+4` points at 128 random bytes less `0` (placed); the word at `p58+0` points at 128 random bytes less `0` (placed); the word at `p58+4` points at 128 random bytes less `0` (placed); the word at `p59+0` points at 128 random bytes less `0` (placed); the word at `p59+4` points at 128 random bytes less `0` (placed); the word at `p60+0` points at 128 random bytes less `0` (placed); the word at `p60+4` points at 128 random bytes less `0` (placed); the word at `p61+0` points at 128 random bytes less `0` (placed); the word at `p61+4` points at 128 random bytes less `0` (placed); the word at `p62+0` points at 128 random bytes less `0` (placed); the word at `p62+4` points at 128 random bytes less `0` (placed); the word at `p63+0` points at 128 random bytes less `0` (placed); the word at `p63+4` points at 128 random bytes less `0` (placed); the word at `p6+0` points at 256 random bytes less `0` (placed); the word at `p13+0` points at 256 random bytes less `0` (placed); the word at `p20+0` points at 256 random bytes less `0` (placed); stub 0 answered from [1, 1, 1, 0] (90%); stub 1 answered from [0, 0, 0, 0, 0, 1] (90%); stub 1 answers 8 values in call order, printed after the answers; stub 2 answers 3 values in call order, printed after the answers; stub 2 answers 0..3 every row; stub 3 answers 1..20 every row; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 3 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the recorded calls one token a call, in call order, each headed `sIDX:` (the callee); 8 rows 0x6eec apart at `@0xe3a390`, slices +0x0:0x4, +0x3f8:0x14, +0x7e4:0x4, +0x9e0:0x4, +0x6dd4:0x4, +0x6e20:0x4, +0x6eb8:0x4 (`p` windows after the pointed buffers, before the places); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 4000 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_army_release_mustering.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (line, calls) = line.split_once(" # ").expect("`<row> # <calls>`");
        let want_log: Vec<&str> = calls.split_whitespace().collect();
        let (lhs, rhs) = line.split_once(" -> ").expect("a row is `<args…> | <windows> -> <eax> | <windows>`");
        let (args, before) = lhs.split_once(" | ").expect("`<args> | <before>`");
        let (eax, after) = rhs.split_once(" | ").expect("`<eax> | <after>`");
        let a: Vec<i32> = args.split_whitespace().map(|s| s.parse().expect("an integer")).collect();
        let mut wins: Vec<Vec<u8>> = before.trim().split(';').map(unhex).collect();
        let mut m = wins.remove(0);
        let mut p: Vec<Vec<u8>> = wins.split_off(1);
        let mut c: Vec<Vec<u8>> = wins;
        let want_eax: i64 = eax.trim().parse().expect("an integer");
        let mut log: Vec<String> = Vec::new();
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            let _ = (&a, &m, &c, &p);
            // m the army (`+0x94` the leader, `+0x20` its city, `+0x18` the size, `+0x24` a flag, `+8` handed
            // to `num_armies`); c0 the game record (`+0x32` rush rules); p[7k..7k+7] leader k's slices (flag word,
            // `+0x3f8..0x40c`, `+0x7e4`, `+0x9e0`, `+0x6dd4`, `+0x6e20`, `+0x6eb8`); p[64 + 2k + j] its cities;
            // p[80 + k] the record `+0x6eb8` names. a[0] `war_allowed`, a[3] `num_armies`, a[4..12]
            // `is_ally`'s answers, a[12..15] `get_diff`'s, in call order. Recorded in call order.
            let rdw = |w: &[u8], o: usize| {
                i32::from_le_bytes(w[o..o + 4].try_into().expect("four bytes"))
            };
            let ld = usize::from(u16::from_le_bytes([m[0x94], m[0x95]]));
            let n = rdw(m, 0x18);
            let mut calls: Vec<String> = Vec::new();
            let ret: i64 = 'out: {
                let city = &p[64 + 2 * ld + rdw(m, 0x20) as usize];
                if i32::from(city[0x5e] as i8) != ld as i32 || city[4] & 1 == 0 {
                    break 'out 1;
                }
                if c[0][0x32] != 0 {
                    calls.push("s0:c0+0x0".to_string());
                    if a[0] == 0 {
                        break 'out 0;
                    }
                }
                if rdw(&p[7 * ld], 0) & 8 != 0 || n < 2 {
                    break 'out 0;
                }
                let mut seen = 0;
                for k in 0..8 {
                    if rdw(&p[7 * k], 0) & 1 == 0 {
                        continue;
                    }
                    calls.push(format!("s1:p{}+0x0,{k}", 7 * ld));
                    let ally = a[4 + seen.min(7)];
                    seen += 1;
                    if ally == 0 {
                        continue;
                    }
                    for j in 0..rdw(&p[7 * k + 1], 0x10).max(0) as usize {
                        if p[64 + 2 * k + j][4] & 3 == 3 && (k == ld || n >= 5) {
                            break 'out 1;
                        }
                    }
                }
                if n < rdw(&p[7 * ld + 1], 0) || n < 5 {
                    break 'out 0;
                }
                if rdw(m, 0x24) != 0 {
                    break 'out 1;
                }
                let v6 = rdw(&p[7 * ld + 2], 0);
                if (v6 * 7) / 8 <= rdw(&p[7 * ld + 3], 0) || v6 / 8 <= n {
                    break 'out 1;
                }
                calls.push(format!("s3:m+0x0,{ld},1,{}", rdw(m, 8)));
                if v6 / (a[3] * 2) <= n {
                    break 'out 1;
                }
                for (i, (want, over)) in [(0, 6), (1, 10), (2, 12)].into_iter().enumerate() {
                    calls.push(format!("s2:p{}+0x0", 7 * ld));
                    if a[12 + i] == want && n > over {
                        break 'out 1;
                    }
                }
                let kind = rdw(&p[7 * ld + 4], 0);
                if kind == -1 {
                    break 'out i64::from(n > 0x20);
                }
                let u = rdw(&p[80 + ld], 0xe8) ^ 0x63187;
                if kind == 1 {
                    if u < 2 {
                        break 'out 1;
                    } else if u < 3 {
                        break 'out i64::from(n > 0xf);
                    } else if u < 5 {
                        break 'out i64::from(n > 0x14);
                    }
                } else if rdw(&p[7 * ld + 5], 0) != 0 {
                    if u < 2 {
                        break 'out 1;
                    } else if u < 3 {
                        break 'out i64::from(n > 0xf);
                    }
                }
                i64::from(n > 0x19)
            };
            if calls.is_empty() {
                log.push("-".to_string());
            }
            log.extend(calls);
            ret
        };
        assert_eq!(got, want_eax, "release_mustering — eax, row `{line}`");
        assert_eq!(log, want_log, "release_mustering — the recorded calls, row `{line}`");
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "release_mustering — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "release_mustering — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[2 + i], "release_mustering — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 4000, "only {n} rows");
    eprintln!("Army::release_mustering@006f87c0: {n} rows agree");
}
