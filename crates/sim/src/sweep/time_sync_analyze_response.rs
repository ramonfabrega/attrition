//! `TimeSync::analyze_response@00954b90` under the emulator against the port (item 2921; tools/sweep_gen.py, obj with windows).

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

/// `TimeSync::analyze_response@00954b90`, run by `tools/emu/sweep_time_sync_analyze_response.py` under unicorn
/// (tools/emu): 16 random bytes of object, 2 argument(s), the global cell 0x00c06378 points at 32 random bytes; 4 random bytes at 0x00c8984c; 8 random bytes at 0x00c8970c; 96 random bytes at 0x00eb1360; the import cell 0x00ac5460 holds a stub (0 args); callee 0x00954c80 (1 args) stubbed; callee 0x00a3ae30 (10 args) stubbed; stub 0 answered from [0, 1, 1000, 2147483647] (90%); stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_time_sync_analyze_response.py") else {
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
        let mut p: Vec<Vec<u8>> = wins.split_off(4);
        let mut c: Vec<Vec<u8>> = wins;
        let want_eax: i64 = eax.trim().parse().expect("an integer");
        let mut log: Vec<String> = Vec::new();
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            let _ = (&a, &m, &c, &p);
            // `param_1 <= ping_times_sorted[12]` (unsigned): `time = param_1/2 + param_2`, `base_time = timeGetTime()`,
            // the log's flag bit 3 set round `Log::say` and cleared after (so cleared), `game_log`'s detail put back unless
            // negative (then 1, what the say left). Either way `add_ping(param_1)`, its `ecx` `this` or the log's.
            let p1 = a[0] as u32;
            let ping = u32::from_le_bytes([c[1][0], c[1][1], c[1][2], c[1][3]]);
            if p1 <= ping {
                let time = (p1 >> 1).wrapping_add(a[1] as u32);
                c[2][0..4].copy_from_slice(&time.to_le_bytes());
                c[2][4..8].copy_from_slice(&a[2].to_le_bytes());
                let flags = rd(&c[3], 4) & !8;
                wr(&mut c[3], 4, flags);
                if rd(&c[3], 0x4c) < 0 {
                    wr(&mut c[3], 0x4c, 1);
                }
                log.push(format!("c3+0x0,{}", a[0]));
                log.push(format!(
                    "c3+0x0,{},15418236,{},{},0,0,0,0,0,2",
                    rd(&c[0], 0x10).wrapping_add(0x21458),
                    time as i32,
                    a[2]
                ));
            } else {
                log.push(format!("m+0x0,{}", a[0]));
                log.push("-".into());
            }
            1
        };
        assert_eq!(got, want_eax, "analyze_response — eax, row `{line}`");
        assert_eq!(log, want_log, "analyze_response — the recorded calls, row `{line}`");
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "analyze_response — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "analyze_response — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[5 + i], "analyze_response — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("TimeSync::analyze_response@00954b90: {n} rows agree");
}
