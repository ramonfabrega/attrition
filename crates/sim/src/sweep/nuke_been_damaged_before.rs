//! `Nuke::been_damaged_before@00925c20` under the emulator against the port (item 2149; tools/sweep_gen.py, obj with windows).

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

/// `Nuke::been_damaged_before@00925c20`, run by `tools/emu/sweep_nuke_been_damaged_before.py` under unicorn
/// (tools/emu): 256 random bytes of object, 3 argument(s), the word at `m+0xdc` points at 4 random bytes less `a[2]*4` (placed); the word at `p0+0x0` points at 20 random bytes less `0` (placed); the word at `p1+0x10` points at 24 random bytes less `0` (placed); the word at `m+0xf8` points at 4 random bytes less `a[2]*4` (placed); the word at `p3+0x0` points at 20 random bytes less `0` (placed); the word at `p4+0x10` points at 24 random bytes less `0` (placed); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_nuke_been_damaged_before.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
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
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            let _ = (&a, &m, &c, &p);
            {
                let n = rd(&p[1], 4);
                let mut r = 0;
                for k in 0..n.max(0) as usize {
                    if rd(&p[2], 4 * k) == a[1] && rd(&p[5], 4 * k) == a[0] {
                        r = 1;
                        break;
                    }
                }
                r
            }
        };
        assert_eq!(got, want_eax, "been_damaged_before — eax, row `{line}`");
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(
            m, want[0],
            "been_damaged_before — this's bytes, row `{line}`"
        );
        for (i, w) in c.iter().enumerate() {
            assert_eq!(
                w,
                &want[1 + i],
                "been_damaged_before — window c[{i}], row `{line}`"
            );
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(
                w,
                &want[1 + i],
                "been_damaged_before — window p[{i}], row `{line}`"
            );
        }
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("Nuke::been_damaged_before@00925c20: {n} rows agree");
}
