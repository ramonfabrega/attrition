//! `Armies::find_aggressive_army@006f2e10` under the emulator against the port (item 2247; tools/sweep_gen.py, obj with windows).

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

/// `Armies::find_aggressive_army@006f2e10`, run by `tools/emu/sweep_armies_find_aggressive_army.py` under unicorn
/// (tools/emu): 32 random bytes of object, 1 argument(s), the global cell 0x00cae5fc points at 1024 random bytes; the global cell 0x00c06188 points at 320 random bytes; the word at `0xc09710+a[0]*0x1c` points at 64 random bytes less `0` (placed); the word at `p0+0` points at 64 random bytes less `0` (placed); the word at `p0+4` points at 64 random bytes less `0` (placed); the word at `p0+8` points at 64 random bytes less `0` (placed); the word at `p0+12` points at 64 random bytes less `0` (placed); the word at `p0+16` points at 64 random bytes less `0` (placed); the word at `p0+20` points at 64 random bytes less `0` (placed); the word at `p0+24` points at 64 random bytes less `0` (placed); the word at `p0+28` points at 64 random bytes less `0` (placed); the word at `p0+32` points at 64 random bytes less `0` (placed); the word at `p0+36` points at 64 random bytes less `0` (placed); the word at `p0+40` points at 64 random bytes less `0` (placed); the word at `p0+44` points at 64 random bytes less `0` (placed); the word at `p0+48` points at 64 random bytes less `0` (placed); the word at `p0+52` points at 64 random bytes less `0` (placed); the word at `p0+56` points at 64 random bytes less `0` (placed); the word at `p0+60` points at 64 random bytes less `0` (placed); the word at `c1+0x134` points at 1792 random bytes less `0` (placed); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_armies_find_aggressive_army.py") else {
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
        let mut p: Vec<Vec<u8>> = wins.split_off(2);
        let mut c: Vec<Vec<u8>> = wins;
        let want_eax: i64 = eax.trim().parse().expect("an integer");
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            let _ = (&a, &m, &c, &p);
            {
                // `who` is `a[0]`; its sixteen armies are `p[1..17]` (`+0` live,
                // `+4` status, `+0x14` captains, `+0x38`/`+0x3c` the position),
                // the world `c[1]` and its cell records `p[17]`.
                let cw = rd(&c[1], 0);
                let found = (0..16).find(|&k| {
                    let r = &p[1 + k];
                    let (x, y) = (
                        rd(&c[0], 4 * (rd(r, 0x38) >> 8) as usize),
                        rd(&c[0], 4 * (rd(r, 0x3c) >> 8) as usize),
                    );
                    let owner = i32::from(p[17][((y * cw + x) * 0x1c + 0xf) as usize] as i8);
                    let live = i16::from_le_bytes([r[0], r[1]]) != 0;
                    crate::army::aggressive(live, rd(r, 0x14), i32::from(r[4]), owner, a[0])
                });
                found.map_or(-1, |k| k as i64)
            }
        };
        assert_eq!(got, want_eax, "find_aggressive_army — eax, row `{line}`");
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(
            m, want[0],
            "find_aggressive_army — this's bytes, row `{line}`"
        );
        for (i, w) in c.iter().enumerate() {
            assert_eq!(
                w,
                &want[1 + i],
                "find_aggressive_army — window c[{i}], row `{line}`"
            );
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(
                w,
                &want[3 + i],
                "find_aggressive_army — window p[{i}], row `{line}`"
            );
        }
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("Armies::find_aggressive_army@006f2e10: {n} rows agree");
}
