//! `Unit::must_walk@0060d040` under the emulator against the port (item 2495; tools/sweep_gen.py, obj with windows).

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

/// `Unit::must_walk@0060d040`, run by `tools/emu/sweep_unit_must_walk.py` under unicorn
/// (tools/emu): 256 random bytes of object, 1 argument(s), argument 0 names 8 random bytes; vtable slot +0x0 (2 args) stubbed; the stubbed vtable laid at `p0+0`; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 0 prints the 1 bytes its argument 0 points at, hex, after the record (`/` a call); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_unit_must_walk.py") else {
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
                let flag = if rd(&p[0], 4) != 0 {
                    (a[0] as u32 >> 24) as u8
                } else if rd(m, 0x18) != 0
                    || m[8] & 1 != 0
                    || m[0x32] | m[0x33] != 0
                    || m[0x8f] & 0x80 == 0
                {
                    1
                } else {
                    0
                };
                log.push("p0+0x0,sp-0x1,2147483392".into());
                log.push(format!("{flag:02x}"));
                i64::from(flag)
            }
        };
        assert_eq!(got, want_eax, "must_walk — eax, row `{line}`");
        assert_eq!(
            log, want_log,
            "must_walk — the recorded calls, row `{line}`"
        );
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "must_walk — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "must_walk — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "must_walk — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("Unit::must_walk@0060d040: {n} rows agree");
}
