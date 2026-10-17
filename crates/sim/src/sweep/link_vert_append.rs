//! `LinkVert::append@008821e0` under the emulator against the port (item 3038; tools/sweep_gen.py, obj with windows).

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

/// `LinkVert::append@008821e0`, run by `tools/emu/sweep_link_vert_append.py` under unicorn
/// (tools/emu): 16 random bytes of object, 1 argument(s), `p` windows in order: p0 argument 0; 4 random bytes at 0x00ee1232; argument 0 names 8 random bytes; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 600 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_link_vert_append.py") else {
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
            // A vertex links to at most two neighbours, `prev` (+4) then `next` (+0); the new one is
            // hung on whichever is empty and points back at `this` (the object's base, 0x20000000). With
            // both taken only the (muted) error report runs. The answer is the argument on the two linking arms, 0 on the third.
            const THIS: i32 = 0x2000_0000;
            if rd(m, 4) == 0 {
                wr(m, 4, a[0]);
                wr(&mut p[0], 0, THIS);
                i64::from(a[0])
            } else if rd(m, 0) == 0 {
                wr(m, 0, a[0]);
                wr(&mut p[0], 4, THIS);
                i64::from(a[0])
            } else {
                0
            }
        };
        assert_eq!(got, want_eax, "append — eax, row `{line}`");
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "append — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "append — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[2 + i], "append — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 600, "only {n} rows");
    eprintln!("LinkVert::append@008821e0: {n} rows agree");
}
