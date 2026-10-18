//! `Roads::generate@00894350` under the emulator against the port (item 3049; tools/sweep_gen.py, obj with windows).

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

/// `Roads::generate@00894350`, run by `tools/emu/sweep_roads_generate.py` under unicorn
/// (tools/emu): 16 random bytes of object, 0 argument(s), `p` windows in order: p0 the place; the global cell 0x00c06238 points at 1520 random bytes; callee 0x00849c40 (0 args) stubbed; callee 0x0088f4b0 (1 args) stubbed; callee 0x004a3a50 (0 args) stubbed; the word at `c0+0x5d0` points at 1280 random bytes less `0` (placed); stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 1200 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_roads_generate.py") else {
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
            // The roads cell `c0`: each of its `+0x5c4` render helpers (0x13c apart from the array at +0x5d0,
            // `p0`) is cleared (stub 0, ecx = the helper); a non-zero `+0xe4` adds roads (stub 1, ecx = the
            // cell, argument 0); a non-zero `+0x5d8` cleans the debug array (stub 2, ecx = c0+0x5e0) and
            // zeroes `+0x5e4`.
            let n = rd(&c[0], 0x5c4).max(0);
            let clears: Vec<String> = (0..n).map(|k| format!("p0+{:#x}", k * 0x13c)).collect();
            log.push(if clears.is_empty() {
                "-".to_string()
            } else {
                clears.join("/")
            });
            log.push(if rd(&c[0], 0xe4) != 0 {
                "c0+0x0,0".to_string()
            } else {
                "-".to_string()
            });
            if rd(&c[0], 0x5d8) != 0 {
                log.push("c0+0x5e0".to_string());
                wr(&mut c[0], 0x5e4, 0);
            } else {
                log.push("-".to_string());
            }
            want_eax
        };
        let _ = (got, want_eax);
        assert_eq!(log, want_log, "generate — the recorded calls, row `{line}`");
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "generate — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "generate — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[2 + i], "generate — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 1200, "only {n} rows");
    eprintln!("Roads::generate@00894350: {n} rows agree");
}
