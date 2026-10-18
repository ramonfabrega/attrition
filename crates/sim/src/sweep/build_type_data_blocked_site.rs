//! `BuildTypeData::blocked_site@00636a50` under the emulator against the port (item 3259; tools/sweep_gen.py, obj with windows).

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

/// `BuildTypeData::blocked_site@00636a50`, run by `tools/emu/sweep_build_type_data_blocked_site.py` under unicorn
/// (tools/emu): 576 random bytes of object, 5 argument(s), `p` windows in order: p0 argument 4, p1..p32 sparse table 0 (8 rows × 4 slice(s), row-major); the global cell 0x00c061d0 points at 16 random bytes; the global cell 0x00cae5fc points at 2048 random bytes; the global cell 0x00c061e8 points at 2304 random bytes; argument 4 names 4 random bytes; vtable slot +0x60 (2 args) stubbed; vtable slot +0x64 (0 args) stubbed; callee 0x00636db0 (4 args) stubbed; callee 0x006b53f0 (3 args) stubbed; callee 0x006375b0 (7 args) stubbed; slot +0x64 laid as 0x00472bc0 or its stub, a flag after the answers; stub 0 answered from [0, 1] (90%); stub 1 answered from [0, 1] (90%); stub 2 answered from [0, 0, 0, 0, 34, 23, 32, 31, 36, 24, 20, 26, 27, 46, 45, 14, 4, 5] (90%); stub 3 answered from [0, 1] (90%); stub 4 answered from [0, 34, 23, 32, 31, 36, 24, 20, 26, 27, 46, 45, 14, 4, 5] (90%); stub 0 answers 2 values in call order, printed after the answers; stub 1 answers 1 values in call order, printed after the answers; stub 2 answers 25 values in call order, printed after the answers; stub 3 answers 25 values in call order, printed after the answers; stub 4 answers 1 values in call order, printed after the answers; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 3 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 4 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 3 prints the 4 bytes its argument 0 points at, hex, after the record (`/` a call); stub 3 prints the 4 bytes its argument 1 points at, hex, after the record (`/` a call); 8 rows 0x6eec apart at `0xc061e0`, slices +0x0:0x4, +0x8:0x4, +0x74:0x20, +0x410:0x4 (`p` windows after the pointed buffers, before the places); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 3000 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_build_type_data_blocked_site.py") else {
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
        let mut p: Vec<Vec<u8>> = wins.split_off(3);
        let mut c: Vec<Vec<u8>> = wins;
        let want_eax: i64 = eax.trim().parse().expect("an integer");
        let mut log: Vec<String> = Vec::new();
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            let _ = (&a, &m, &c, &p);
            // `BuildTypeData::blocked_site@00636a50(x, y, who, exclude, &slots)`. The crate's
            // `Sim::blocked_site` takes a whole `Sim` whose own `blocked_tcoord`,
            // `blocked_location` and `was_seen` would have to be driven to the stubs' answers
            // tile by tile, so this is the listing's port (Loop 3237: out of reach, inline).
            // a[0..5] the arguments, a[5..10] the stubs' one answer (unread), a[10..12] the
            // `is` slot's answers (stub 0), a[12] the +0x64 slot's (stub 1), a[13..38] tcoord's
            // (stub 2), a[38..63] was_seen's (stub 3), a[63] blocked_location's (stub 4),
            // a[64] whether slot +0x64 was laid as `is_city`. c1 `div_3_table`, c2 the game
            // (+0x550 frame, +0x821 semaphore), p[1 + 4k + 0..4] leader k's flags, +8, +0x74..,
            // +0x410 slices.
            let (x, y, who, excl) = (a[0], a[1], a[2], a[3]);
            let dv = |k: i32| rd(&c[1], 4 * usize::try_from(k).expect("table index"));
            let (xs, ys) = (rd(m, 0x234), rd(m, 0x238));
            let sem = c[2][0x821] & 8 != 0;
            let frame = rd(&c[2], 0x550);
            let mut bx = dv(x >> 6) * 0xc0 + 0x60;
            if xs & 1 == 0 {
                bx -= 0x60;
            }
            let mut by = dv(y >> 6) * 0xc0 + 0x60;
            if ys & 1 == 0 {
                by -= 0x60;
            }
            let (tx0, ty0) = (dv(bx >> 6) - (xs >> 1), dv(by >> 6) - (ys >> 1));
            let refused = |r: i32| {
                matches!(
                    r,
                    0x20 | 0x1f | 0x24 | 0x18 | 0x14 | 0x17 | 0x1a | 0x1b | 0x2e | 0x2d
                )
            };
            let (mut n0, mut n2, mut n3) = (0usize, 0usize, 0usize);
            let (mut l0, mut l1, mut l2, mut l3, mut l4): (
                Vec<String>,
                Vec<String>,
                Vec<String>,
                Vec<String>,
                Vec<String>,
            ) = (Vec::new(), Vec::new(), Vec::new(), Vec::new(), Vec::new());
            let (mut d0, mut d1): (Vec<String>, Vec<String>) = (Vec::new(), Vec::new());
            let hex = |v: i32| {
                v.to_le_bytes()
                    .iter()
                    .map(|b| format!("{b:02x}"))
                    .collect::<String>()
            };
            let leader = |k: usize, j: usize| &p[1 + 4 * k + j];
            let mut is = |t: i32| {
                let r = a[10 + n0.min(1)] != 0;
                n0 += 1;
                l0.push(format!("m+0x0,{t},0"));
                r
            };
            let city = if a[64] != 0 {
                is(0x19e)
            } else {
                l1.push("m+0x0".to_string());
                a[12] != 0
            };
            let mut early: Option<i32> = None;
            if city && who >= 0 {
                let who = usize::try_from(who).expect("a player");
                // The counter the listing compares with the leader's `+8` is the leader's own
                // index: the `xor eax, eax` is before the loop's back-edge, not in it.
                for i in 0..8usize {
                    if leader(i, 0)[0] & 1 != 0 {
                        let f8 = rd(leader(who, 1), 0);
                        let tied = i32::try_from(i).expect("i") == f8
                            || (rd(leader(who, 2), 4 * i) == 2
                                && rd(leader(i, 2), 4 * usize::try_from(f8).expect("f8")) == 2);
                        if tied
                            && leader(who, 0)[0] & 4 != 0
                            && frame - 0x4b < rd(leader(i, 3), 0)
                            && !sem
                        {
                            early = Some(0x2d);
                            break;
                        }
                    }
                }
            }
            let result: i32 = 'done: {
                if let Some(r) = early {
                    break 'done r;
                }
                let (mut first, mut unseen) = (0i32, 0i32);
                for tx in tx0..tx0 + xs {
                    for ty in ty0..ty0 + ys {
                        let r = a[13 + n2.min(24)];
                        n2 += 1;
                        l2.push(format!("m+0x0,{tx},{ty},{who},{excl}"));
                        if r == 0x22 {
                            break 'done 0x22;
                        }
                        if r != 0 && (first == 0 || r == 0x17) && (!sem || !refused(r)) {
                            first = r;
                        }
                        if who >= 0 {
                            let seen = a[38 + n3.min(24)] != 0;
                            n3 += 1;
                            l3.push(format!("c0+0x0,sp-0x20,sp-0x30,{who}"));
                            d0.push(hex(tx >> 1));
                            d1.push(hex(ty >> 1));
                            if !seen && !sem {
                                unseen += 1;
                            }
                        }
                    }
                }
                if first == 0x24 {
                    break 'done 0x24;
                }
                if xs * ys / 2 < unseen && !is(0x1b0) {
                    break 'done 0x24;
                }
                if first == 0xe {
                    break 'done 0xe;
                }
                l4.push(format!("m+0x0,{x},{y},{tx0},{ty0},{who},{excl},p0+0x0"));
                let r = a[63];
                if r == 0 || (sem && refused(r)) {
                    first
                } else {
                    r
                }
            };
            let j = |v: Vec<String>| {
                if v.is_empty() {
                    "-".to_string()
                } else {
                    v.join("/")
                }
            };
            for v in [l0, l1, l2, l3, l4, d0, d1] {
                log.push(j(v));
            }
            i64::from(result)
        };
        assert_eq!(got, want_eax, "blocked_site — eax, row `{line}`");
        assert_eq!(
            log, want_log,
            "blocked_site — the recorded calls, row `{line}`"
        );
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "blocked_site — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(
                w,
                &want[1 + i],
                "blocked_site — window c[{i}], row `{line}`"
            );
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(
                w,
                &want[4 + i],
                "blocked_site — window p[{i}], row `{line}`"
            );
        }
        n += 1;
    }
    assert!(n >= 3000, "only {n} rows");
    eprintln!("BuildTypeData::blocked_site@00636a50: {n} rows agree");
}
