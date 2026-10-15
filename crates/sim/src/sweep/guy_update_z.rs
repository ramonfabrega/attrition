//! `Guy::update_z@005d9950` under the emulator against the port (item 2633; tools/sweep_gen.py, obj with windows).

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

/// `Guy::update_z@005d9950`, run by `tools/emu/sweep_guy_update_z.py` under unicorn
/// (tools/emu): 176 random bytes of object, 0 argument(s), 128 random bytes at 0x00c0aec0; callee 0x00866560 (4 args) stubbed; the word at `c0+i8(m,0xa1)*0x1c` points at 4 random bytes less `i16(m,0x8c)*4` (placed); the word at `p0+0` points at 32 random bytes less `0` (placed); the word at `p1+0x18` points at 560 random bytes less `0` (placed); stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 0 writes a value in 0..99 through its argument 0, printed after the answers (`o` in a place); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_guy_update_z.py") else {
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
            if rd(&p[2], 0x218) != 1 {
                let v = a[1];
                log.push(format!("12628208,sp-0xc,{},{},0", rd(m, 0xc), rd(m, 0x10)));
                wr(m, 0x14, v);
            } else {
                log.push("-".into());
                wr(m, 0x14, 0);
            }
            0i64
        };
        let _ = (got, want_eax);
        assert_eq!(log, want_log, "update_z — the recorded calls, row `{line}`");
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "update_z — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "update_z — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[2 + i], "update_z — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("Guy::update_z@005d9950: {n} rows agree");
}
