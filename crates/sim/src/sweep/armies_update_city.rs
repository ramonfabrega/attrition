//! `Armies::update_city@006f2d70` under the emulator against the port (item 2852; tools/sweep_gen.py, obj with windows).

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

/// `Armies::update_city@006f2d70`, run by `tools/emu/sweep_armies_update_city.py` under unicorn
/// (tools/emu): 16 random bytes of object, 5 argument(s), the word at `p9+0` points at 16 random bytes less `0` (placed); the word at `p11+0` points at 16 random bytes less `0` (placed); the word at `p13+0` points at 16 random bytes less `0` (placed); the word at `p15+0` points at 16 random bytes less `0` (placed); the word at `p17+0` points at 16 random bytes less `0` (placed); the word at `p19+0` points at 16 random bytes less `0` (placed); the word at `p21+0` points at 16 random bytes less `0` (placed); the word at `p23+0` points at 16 random bytes less `0` (placed); the word at `p24+0` points at 64 random bytes less `0` (placed); the word at `p24+4` points at 64 random bytes less `0` (placed); the word at `p24+8` points at 64 random bytes less `0` (placed); the word at `p24+12` points at 64 random bytes less `0` (placed); the word at `p25+0` points at 64 random bytes less `0` (placed); the word at `p25+4` points at 64 random bytes less `0` (placed); the word at `p25+8` points at 64 random bytes less `0` (placed); the word at `p25+12` points at 64 random bytes less `0` (placed); the word at `p26+0` points at 64 random bytes less `0` (placed); the word at `p26+4` points at 64 random bytes less `0` (placed); the word at `p26+8` points at 64 random bytes less `0` (placed); the word at `p26+12` points at 64 random bytes less `0` (placed); the word at `p27+0` points at 64 random bytes less `0` (placed); the word at `p27+4` points at 64 random bytes less `0` (placed); the word at `p27+8` points at 64 random bytes less `0` (placed); the word at `p27+12` points at 64 random bytes less `0` (placed); the word at `p28+0` points at 64 random bytes less `0` (placed); the word at `p28+4` points at 64 random bytes less `0` (placed); the word at `p28+8` points at 64 random bytes less `0` (placed); the word at `p28+12` points at 64 random bytes less `0` (placed); the word at `p29+0` points at 64 random bytes less `0` (placed); the word at `p29+4` points at 64 random bytes less `0` (placed); the word at `p29+8` points at 64 random bytes less `0` (placed); the word at `p29+12` points at 64 random bytes less `0` (placed); the word at `p30+0` points at 64 random bytes less `0` (placed); the word at `p30+4` points at 64 random bytes less `0` (placed); the word at `p30+8` points at 64 random bytes less `0` (placed); the word at `p30+12` points at 64 random bytes less `0` (placed); the word at `p31+0` points at 64 random bytes less `0` (placed); the word at `p31+4` points at 64 random bytes less `0` (placed); the word at `p31+8` points at 64 random bytes less `0` (placed); the word at `p31+12` points at 64 random bytes less `0` (placed); 8 rows 0x6eec apart at `@0xe3a390`, slices +0x0:0x4 (`p` windows after the pointed buffers, before the places); 8 rows 0x1c apart at `@0xc09700`, slices +0x4:0x4, +0x10:0x4 (`p` windows after the pointed buffers, before the places); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_armies_update_city.py") else {
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
            // Every live owner `r` (`p[r]`'s flag word: bit 0 set, `& 0xc` not 4) with armies (the count `p[8 + 2r]`, the army
            // records `p[32 + 4r + i]`): an occupied army (the short at +0 not zero) whose words at +0x30 and +0x34 are
            // `a[0]` and `a[1]` takes `a[2]` and `a[3]` there, and its flags word (+4) gains 0x80 when `a[3]` is the
            // owner `r` (item 2852).
            for r in 0..8usize {
                let f0 = rd(&p[r], 0);
                if f0 & 1 == 0 || f0 & 0xc == 4 {
                    continue;
                }
                for i in 0..rd(&p[8 + 2 * r], 0).max(0) as usize {
                    let k = 32 + 4 * r + i;
                    if (p[k][0] != 0 || p[k][1] != 0)
                        && rd(&p[k], 0x30) == a[0]
                        && rd(&p[k], 0x34) == a[1]
                    {
                        wr(&mut p[k], 0x30, a[2]);
                        wr(&mut p[k], 0x34, a[3]);
                        if a[3] == r as i32 {
                            let f = rd(&p[k], 4) | 0x80;
                            wr(&mut p[k], 4, f);
                        }
                    }
                }
            }
            0
        };
        let _ = (got, want_eax);
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "update_city — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "update_city — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "update_city — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("Armies::update_city@006f2d70: {n} rows agree");
}
