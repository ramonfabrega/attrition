//! `CliffsOut::get_mining_size@008a6da0` under the emulator against the port (item 2170; tools/sweep_gen.py, obj with windows).

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

/// `CliffsOut::get_mining_size@008a6da0`, run by `tools/emu/sweep_cliffs_out_get_mining_size.py` under unicorn
/// (tools/emu): 8 random bytes of object, 2 argument(s), the global cell 0x00c061d0 points at 4 random bytes; argument 0 names 4 random bytes; argument 1 names 4 random bytes; the word at `0xc06218` points at 4 random bytes less `0x6ab0` (placed); the word at `p2+0` points at 4 random bytes less `(i32(p[1],0)*i32(c[0],0)+i32(p[0],0))*4` (placed); the word at `p3+0` points at 4 random bytes less `0x6a` (placed); the word at `0xe88ee4` points at 4 random bytes less `i16(p[4],0)*4` (placed); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_cliffs_out_get_mining_size.py") else {
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
                let s = i16::from_le_bytes([p[4][0], p[4][1]]);
                if s == -1 { 0 } else { i64::from(rd(&p[5], 0)) }
            }
        };
        assert_eq!(got, want_eax, "get_mining_size — eax, row `{line}`");
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "get_mining_size — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(
                w,
                &want[1 + i],
                "get_mining_size — window c[{i}], row `{line}`"
            );
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(
                w,
                &want[2 + i],
                "get_mining_size — window p[{i}], row `{line}`"
            );
        }
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("CliffsOut::get_mining_size@008a6da0: {n} rows agree");
}
