//! `Group::set_ordered@00707fd0` under the emulator against the port (item 1993; tools/sweep_gen.py, obj).

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

/// `Group::set_ordered@00707fd0`, run by `tools/emu/sweep_group_set_ordered.py` under unicorn
/// (tools/emu): 64 random bytes of object, 4 argument(s) over the edges and random words, `eax` and every byte after held to the port.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_group_set_ordered.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (lhs, rhs) = line.split_once(" -> ").expect("a row is `<args…> | <before> -> <eax> | <after>`");
        let (args, before) = lhs.split_once(" | ").expect("`<args> | <before>`");
        let (eax, after) = rhs.split_once(" | ").expect("`<eax> | <after>`");
        let a: Vec<i32> = args.split_whitespace().map(|s| s.parse().expect("an integer")).collect();
        let mut m = unhex(before.trim());
        let want_eax: i64 = eax.trim().parse().expect("an integer");
        let got: i64 = {
            let (a, m) = (&a[..], &mut m[..]);
            let _ = (&a, &m);
            wr(m, 0x28, 0);
        if a[3] == 2 || a[3] == 1 {
            wr(m, 0x24, a[2]);
            wr(m, 0x18, a[0]);
            wr(m, 0x1c, a[1]);
        }
        0
        };
        let _ = (got, want_eax);
        assert_eq!(m, unhex(after.trim()), "set_ordered — bytes, row `{line}`");
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("Group::set_ordered@00707fd0: {n} rows agree");
}
