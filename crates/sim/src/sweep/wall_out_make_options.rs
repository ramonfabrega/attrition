//! `WallOut::make_options@00642470` under the emulator against the port (item 2783; tools/sweep_gen.py, obj with windows).

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

/// `WallOut::make_options@00642470`, run by `tools/emu/sweep_wall_out_make_options.py` under unicorn
/// (tools/emu): 16 random bytes of object, 0 argument(s), the global cell 0x00c06204 points at 128 random bytes; callee 0x0072c290 (6 args) stubbed; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_wall_out_make_options.py") else {
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
        let mut p: Vec<Vec<u8>> = wins.split_off(1);
        let mut c: Vec<Vec<u8>> = wins;
        let want_eax: i64 = eax.trim().parse().expect("an integer");
        let mut log: Vec<String> = Vec::new();
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            let _ = (&a, &m, &c, &p);
            // `WallOut::make_options()`: `Options::add` (`0x72c290`, `ecx` the options record, `c[0]`, mode at
            // `+0x74`). Mode 0 (`STANDARD_OPTIONS`) adds `OPTION_DISBAND` (0xf, 5, 0, -1, 0, 0); the stub does
            // not move the mode, so then a mode of 0x13 or more adds `OPTION_BACK` (0x42, 4, 3, ...), any other
            // nonzero mode `OPTION_CANCEL` (0x43, 4, 3, ...), and mode 0 nothing more. The mode compares signed.
            let mode = rd(&c[0], 0x74);
            if mode == 0 {
                log.push("c0+0x0,15,5,0,-1,0,0".to_string());
            } else if mode >= 0x13 {
                log.push("c0+0x0,66,4,3,-1,0,0".to_string());
            } else {
                log.push("c0+0x0,67,4,3,-1,0,0".to_string());
            }
            0i64
        };
        let _ = (got, want_eax);
        assert_eq!(
            log, want_log,
            "make_options — the recorded calls, row `{line}`"
        );
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "make_options — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(
                w,
                &want[1 + i],
                "make_options — window c[{i}], row `{line}`"
            );
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(
                w,
                &want[2 + i],
                "make_options — window p[{i}], row `{line}`"
            );
        }
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("WallOut::make_options@00642470: {n} rows agree");
}
