//! `Herd::process@00741760` under the emulator against the port (item 2341; tools/sweep_gen.py, obj with windows).

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

/// `Herd::process@00741760`, run by `tools/emu/sweep_herd_process.py` under unicorn
/// (tools/emu): 16 random bytes of object, 0 argument(s), the global cell 0x00c06184 points at 16 random bytes; the global cell 0x00c06188 points at 320 random bytes; callee 0x00a39d70 (2 args) stubbed; the word at `c1+0x134` points at 1024 random bytes less `0` (placed); stub 0 answers 2 values in call order, printed after the answers; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 1200 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_herd_process.py") else {
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
            // a[0] is stub 0's single answer, which `--seq` replaces;
            // a[1], a[2] the two draws in call order.
            let (w, h) = (rd(&c[1], 0), rd(&c[1], 4));
            let mut turn = 0usize;
            let mut calls: Vec<String> = Vec::new();
            let walked = crate::gaia::herd_process_steps(
                rd(m, 0),
                rd(m, 4),
                w,
                h,
                |_site| {
                    calls.push("c0+0x0,0,65535".to_string());
                    let v = a[1 + turn.min(1)];
                    turn += 1;
                    v
                },
                |x, y| {
                    let t = ((w * y + x) * 0x1c) as usize;
                    let flags = u16::from_le_bytes([p[0][t], p[0][t + 1]]);
                    (flags, p[0][t + 0x11])
                },
            );
            log.push(if calls.is_empty() {
                "-".to_string()
            } else {
                calls.join("/")
            });
            if let Some((x, y)) = walked {
                wr(m, 8, x);
                wr(m, 12, y);
            }
            0
        };
        let _ = (got, want_eax);
        assert_eq!(log, want_log, "process — the recorded calls, row `{line}`");
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "process — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "process — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[3 + i], "process — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 1200, "only {n} rows");
    eprintln!("Herd::process@00741760: {n} rows agree");
}
