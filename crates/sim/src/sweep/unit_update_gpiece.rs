//! `Unit::update_gpiece@005e2920` under the emulator against the port (item 2314; tools/sweep_gen.py, obj with windows).

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

/// `Unit::update_gpiece@005e2920`, run by `tools/emu/sweep_unit_update_gpiece.py` under unicorn
/// (tools/emu): 256 random bytes of object, 0 argument(s), callee 0x005d8530 (0 args) stubbed; the word at `m+0x18` points at 784 random bytes less `0` (placed); the word at `m+0xf4` points at 32 random bytes less `0` (placed); the word at `p1+0` points at 212 random bytes less `0` (placed); the word at `p1+4` points at 212 random bytes less `0` (placed); the word at `p1+8` points at 212 random bytes less `0` (placed); the word at `p1+12` points at 212 random bytes less `0` (placed); the word at `p1+16` points at 212 random bytes less `0` (placed); the word at `p1+20` points at 212 random bytes less `0` (placed); the word at `p1+24` points at 212 random bytes less `0` (placed); the word at `p1+28` points at 212 random bytes less `0` (placed); stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_unit_update_gpiece.py") else {
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
                let walk = crate::anim::gpiece_walk(
                    m[0xb5] as i8,
                    rd(&p[0], 0x304),
                    rd(&p[0], 0x30c),
                    rd(m, 0xe8),
                );
                let mut calls = Vec::new();
                for k in walk {
                    let g = 2 + k as usize;
                    calls.push(format!("p{g}+0x0"));
                    wr(&mut p[g], 0xd0, 0);
                }
                log.push(if calls.is_empty() {
                    "-".into()
                } else {
                    calls.join("/")
                });
                0
            }
        };
        let _ = (got, want_eax);
        assert_eq!(
            log, want_log,
            "update_gpiece — the recorded calls, row `{line}`"
        );
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "update_gpiece — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(
                w,
                &want[1 + i],
                "update_gpiece — window c[{i}], row `{line}`"
            );
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(
                w,
                &want[1 + i],
                "update_gpiece — window p[{i}], row `{line}`"
            );
        }
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("Unit::update_gpiece@005e2920: {n} rows agree");
}
