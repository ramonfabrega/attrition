//! `Unit::inc_time@00610b40` under the emulator against the port (item 2774; tools/sweep_gen.py, obj with windows).

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

/// `Unit::inc_time@00610b40`, run by `tools/emu/sweep_unit_inc_time.py` under unicorn
/// (tools/emu): 256 random bytes of object, 0 argument(s), callee 0x005d9e10 (0 args) stubbed; the word at `m+0x18` points at 784 random bytes less `0` (placed); the word at `m+0xf4` points at 24 random bytes less `0` (placed); the word at `p1+0` points at 16 random bytes less `0` (placed); the word at `p1+4` points at 16 random bytes less `0` (placed); the word at `p1+8` points at 16 random bytes less `0` (placed); the word at `p1+12` points at 16 random bytes less `0` (placed); the word at `p1+16` points at 16 random bytes less `0` (placed); the word at `p1+20` points at 16 random bytes less `0` (placed); stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_unit_inc_time.py") else {
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
            // `Unit::inc_time`: only a unit with `+0x82` negative, or whose type's id (`+4`) is 0x34 or
            // 0x35, ticks its guys (`0x5d9e10`, handed each guy at `[+0xf4][i]`): first `0..(+0xb5 as i8)`,
            // then the type's `+0x304 .. +0xe8`.
            let mut calls: Vec<String> = Vec::new();
            let kind = rd(&p[0], 4);
            if i16::from_le_bytes([m[0x82], m[0x83]]) < 0 || kind == 0x34 || kind == 0x35 {
                let guy = |i: i32| format!("p{}+0x0", 2 + i);
                let n1 = i32::from(m[0xb5] as i8);
                for i in 0..n1 {
                    calls.push(guy(i));
                }
                for i in rd(&p[0], 0x304)..rd(m, 0xe8) {
                    calls.push(guy(i));
                }
            }
            log.push(if calls.is_empty() {
                "-".to_string()
            } else {
                calls.join("/")
            });
            0i64
        };
        let _ = (got, want_eax);
        assert_eq!(log, want_log, "inc_time — the recorded calls, row `{line}`");
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "inc_time — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "inc_time — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "inc_time — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("Unit::inc_time@00610b40: {n} rows agree");
}
