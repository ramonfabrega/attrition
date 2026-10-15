//! `Armies::find_useful_army@006f2fe0` under the emulator against the port (item 2749; tools/sweep_gen.py, obj with windows).

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

/// `Armies::find_useful_army@006f2fe0`, run by `tools/emu/sweep_armies_find_useful_army.py` under unicorn
/// (tools/emu): 64 random bytes of object, 5 argument(s), 4 random bytes at 0x00e3a390; 4 random bytes at 0x00e4127c; 4 random bytes at 0x00cb4bac; callee 0x006b52e0 (2 args) stubbed; callee 0x0046cff0 (0 args) run, not hooked; the word at `0xcae5fc` points at 1024 random bytes less `0` (placed); the word at `0xc09710+a[0]*0x1c` points at 64 random bytes less `0` (placed); the word at `p1+0` points at 64 random bytes less `0` (placed); the word at `p1+4` points at 64 random bytes less `0` (placed); the word at `p1+8` points at 64 random bytes less `0` (placed); the word at `p1+12` points at 64 random bytes less `0` (placed); the word at `p1+16` points at 64 random bytes less `0` (placed); the word at `p1+20` points at 64 random bytes less `0` (placed); the word at `p1+24` points at 64 random bytes less `0` (placed); the word at `p1+28` points at 64 random bytes less `0` (placed); the word at `p1+32` points at 64 random bytes less `0` (placed); the word at `p1+36` points at 64 random bytes less `0` (placed); the word at `p1+40` points at 64 random bytes less `0` (placed); the word at `p1+44` points at 64 random bytes less `0` (placed); the word at `p1+48` points at 64 random bytes less `0` (placed); the word at `p1+52` points at 64 random bytes less `0` (placed); the word at `p1+56` points at 64 random bytes less `0` (placed); the word at `p1+60` points at 64 random bytes less `0` (placed); stub 0 answered from [0, 1, 2, 3] (90%); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_armies_find_useful_army.py") else {
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
        let mut p: Vec<Vec<u8>> = wins.split_off(3);
        let mut c: Vec<Vec<u8>> = wins;
        let want_eax: i64 = eax.trim().parse().expect("an integer");
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            let _ = (&a, &m, &c, &p);
            {
                // get_tregion (stub 0, hooked) answers a[5]; vector_dist (stub 1) is the original's own code.
                // Records are p[2..18]; the leaders' flag words are c[0] and c[1], `find_dist` is c[2].
                let who = usize::try_from(a[0]).expect("a who");
                let tr = a[5];
                let leader = rd(&c[who], 0) & 0x700 != 0;
                let mut best = 99_999_999;
                let mut found = -1;
                for i in 0..16usize {
                    let r = &p[2 + i];
                    let up = u16::from_le_bytes([r[0], r[1]]) != 0;
                    let div = rd(r, 0x14);
                    if up && div != 0 && ((rd(r, 0x24) == 0 && leader) || tr == rd(r, 8)) {
                        let mut d = crate::world::vector_dist(
                            (a[1] - rd(r, 0x38)).abs(),
                            (a[2] - rd(r, 0x3c)).abs(),
                        );
                        if tr != rd(r, 8) {
                            d *= 3;
                        }
                        if d / div <= best {
                            found = i32::try_from(i).expect("an index");
                            best = d / div;
                        }
                    }
                }
                wr(&mut c[2], 0, best);
                i64::from(found)
            }
        };
        assert_eq!(got, want_eax, "find_useful_army — eax, row `{line}`");
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "find_useful_army — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(
                w,
                &want[1 + i],
                "find_useful_army — window c[{i}], row `{line}`"
            );
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(
                w,
                &want[4 + i],
                "find_useful_army — window p[{i}], row `{line}`"
            );
        }
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("Armies::find_useful_army@006f2fe0: {n} rows agree");
}
