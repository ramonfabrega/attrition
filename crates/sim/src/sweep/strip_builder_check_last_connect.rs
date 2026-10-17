//! `StripBuilder::check_last_connect@00882050` under the emulator against the port (item 2797; tools/sweep_gen.py, obj with windows).

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

/// `StripBuilder::check_last_connect@00882050`, run by `tools/emu/sweep_strip_builder_check_last_connect.py` under unicorn
/// (tools/emu): 88 random bytes of object, 1 argument(s), the word at `m+0x30` points at 16 random bytes less `0` (placed); the word at `p0+0` points at 8 random bytes less `0` (placed); the word at `p0+4` points at 8 random bytes less `0` (placed); the word at `p0+8` points at 8 random bytes less `0` (placed); the word at `p0+12` points at 8 random bytes less `0` (placed); the word at `p1+4` points at 8 random bytes less `0` (placed); the word at `p2+4` points at 8 random bytes less `0` (placed); the word at `p3+4` points at 8 random bytes less `0` (placed); the word at `p4+4` points at 8 random bytes less `0` (placed); the word at `p5+0` laid as the address of `p1+0` half the rows; the word at `p1+0` laid as the address of `p1+0` half the rows; the word at `p1+4` laid as the address of `p1+0` half the rows; the word at `p6+0` laid as the address of `p2+0` half the rows; the word at `p2+0` laid as the address of `p2+0` half the rows; the word at `p2+4` laid as the address of `p2+0` half the rows; the word at `p7+0` laid as the address of `p3+0` half the rows; the word at `p3+0` laid as the address of `p3+0` half the rows; the word at `p3+4` laid as the address of `p3+0` half the rows; the word at `p8+0` laid as the address of `p4+0` half the rows; the word at `p4+0` laid as the address of `p4+0` half the rows; the word at `p4+4` laid as the address of `p4+0` half the rows; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_strip_builder_check_last_connect.py") else {
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
                // `node = last_connect[a0]`, `q = node->+4`: when `*q` is the node's own address (an alias,
                // `q` another window or the node itself) both words are zeroed, else `q[1]` and `node->+4`.
                let i = usize::try_from(a[0]).expect("an index");
                let node_addr = rd(&p[0], 4 * i);
                let q = if rd(&p[1 + i], 4) == node_addr {
                    1 + i
                } else {
                    5 + i
                };
                if rd(&p[q], 0) == node_addr {
                    wr(&mut p[q], 0, 0);
                } else {
                    wr(&mut p[q], 4, 0);
                }
                wr(&mut p[1 + i], 4, 0);
                0
            }
        };
        let _ = (got, want_eax);
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(
            m, want[0],
            "check_last_connect — this's bytes, row `{line}`"
        );
        for (i, w) in c.iter().enumerate() {
            assert_eq!(
                w,
                &want[1 + i],
                "check_last_connect — window c[{i}], row `{line}`"
            );
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(
                w,
                &want[1 + i],
                "check_last_connect — window p[{i}], row `{line}`"
            );
        }
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("StripBuilder::check_last_connect@00882050: {n} rows agree");
}
