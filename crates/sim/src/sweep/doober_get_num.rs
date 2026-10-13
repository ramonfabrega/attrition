//! `Doober::get_num@00846450` under the emulator against the port (item 1954; tools/sweep_gen.py, obj).

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

/// `Doober::get_num@00846450`, run by `tools/emu/sweep_doober_get_num.py` under unicorn
/// (tools/emu): 32 random bytes of object, 4 argument(s) over the edges and random words, `eax` and every byte after held to the port.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_doober_get_num.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (lhs, rhs) = line
            .split_once(" -> ")
            .expect("a row is `<args…> | <before> -> <eax> | <after>`");
        let (args, before) = lhs.split_once(" | ").expect("`<args> | <before>`");
        let (eax, after) = rhs.split_once(" | ").expect("`<eax> | <after>`");
        let a: Vec<i32> = args
            .split_whitespace()
            .map(|s| s.parse().expect("an integer"))
            .collect();
        let mut m = unhex(before.trim());
        let want_eax: i64 = eax.trim().parse().expect("an integer");
        let got: i64 = {
            let (a, m) = (&a[..], &mut m[..]);
            let _ = &a;
            let d = a[3].wrapping_sub(a[2]);
            let n = d.wrapping_abs().wrapping_add(1);
            i64::from(
                a[0].wrapping_mul(a[0])
                    .wrapping_mul(a[1])
                    .wrapping_rem(n)
                    .wrapping_add(a[2]),
            )
        };
        let _ = (got, want_eax);
        assert_eq!(m, unhex(after.trim()), "get_num — bytes, row `{line}`");
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("Doober::get_num@00846450: {n} rows agree");
}
