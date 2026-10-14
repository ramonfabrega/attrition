//! `GroupData::num_valid@0070d7e0` under the emulator against the port (item 2129; tools/sweep_gen.py, obj with windows).

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

/// `GroupData::num_valid@0070d7e0`, run by `tools/emu/sweep_group_data_num_valid.py` under unicorn
/// (tools/emu): 2272 random bytes of object, 0 argument(s), 56 random bytes at 0x00c0ab84; the word at `c0+0` points at 16 random bytes less `0` (placed); the word at `c0+0x1c` points at 16 random bytes less `0` (placed); the word at `p0+0` points at 12 random bytes less `0` (placed); the word at `p0+4` points at 12 random bytes less `0` (placed); the word at `p0+8` points at 12 random bytes less `0` (placed); the word at `p0+0xc` points at 12 random bytes less `0` (placed); the word at `p1+0` points at 12 random bytes less `0` (placed); the word at `p1+4` points at 12 random bytes less `0` (placed); the word at `p1+8` points at 12 random bytes less `0` (placed); the word at `p1+0xc` points at 12 random bytes less `0` (placed); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_group_data_num_valid.py") else {
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
        let mut p: Vec<Vec<u8>> = wins.split_off(1);
        let mut c: Vec<Vec<u8>> = wins;
        let want_eax: i64 = eax.trim().parse().expect("an integer");
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            let _ = (&a, &m, &c, &p);
            {
                let who = m[0x4a] as usize;
                let num = rd(m, 0xc);
                let mut n = 0i64;
                for k in 0..num.max(0) as usize {
                    let o = 0x8cc + 2 * k;
                    let idx = i16::from_le_bytes([m[o], m[o + 1]]) as usize;
                    if p[2 + 4 * who + idx][8] & 1 != 0 {
                        n += 1;
                    }
                }
                n
            }
        };
        assert_eq!(got, want_eax, "num_valid — eax, row `{line}`");
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "num_valid — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "num_valid — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[2 + i], "num_valid — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("GroupData::num_valid@0070d7e0: {n} rows agree");
}
