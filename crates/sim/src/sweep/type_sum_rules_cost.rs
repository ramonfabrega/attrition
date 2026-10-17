//! `Type::sum_rules_cost@006680e0` under the emulator against the port (item 2927; tools/sweep_gen.py, obj with windows).

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

/// `Type::sum_rules_cost@006680e0`, run by `tools/emu/sweep_type_sum_rules_cost.py` under unicorn
/// (tools/emu): 64 random bytes of object, 0 argument(s), the global cell 0x00c061f0 points at 1024 random bytes; vtable slot +0xc (0 args) stubbed; vtable slot +0x14 (0 args) stubbed; vtable slot +0x48 (0 args) stubbed; vtable slot +0x28 (0 args) stubbed; slot +0xc laid as 0x004707e0 or its stub, a flag after the answers; slot +0x14 laid as 0x004707c0 or its stub, a flag after the answers; slot +0x48 laid as 0x00470590 or its stub, a flag after the answers; slot +0x28 laid as 0x004706c0 or its stub, a flag after the answers; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_type_sum_rules_cost.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (lhs, rhs) = line.split_once(" -> ").expect("a row is `<args…> | <windows> -> <eax> | <windows>`");
        let (args, before) = lhs.split_once(" | ").expect("`<args> | <before>`");
        let (eax, after) = rhs.split_once(" | ").expect("`<eax> | <after>`");
        let a: Vec<i32> = args.split_whitespace().map(|s| s.parse().expect("an integer")).collect();
        let mut wins: Vec<Vec<u8>> = before.trim().split(';').map(unhex).collect();
        let mut m = wins.remove(0);
        let mut p: Vec<Vec<u8>> = wins.split_off(1);
        let mut c: Vec<Vec<u8>> = wins;
        let want_eax: i64 = eax.trim().parse().expect("an integer");
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            let _ = (&a, &m, &c, &p);
            {
    // The cost words +0x18..+0x2c summed; the first class test that holds picks the factor of the constants record.
    let sum = (0..6).fold(0i32, |s, k| s.wrapping_add(rd(m, 0x18 + 4 * k)));
    let t = rd(m, 4);
    let k = &c[0];
    let kf = |o: usize| i32::from_le_bytes(k[o..o + 4].try_into().expect("four bytes"));
    // a[0..4] the slots 0xc, 0x14, 0x48, 0x28 as the stub answers; a[4..8] the flags: laid as the class's own test.
    let class = |i: usize, lo: i32, hi: i32| if a[4 + i] != 0 { t > lo && t < hi } else { a[i] != 0 };
    let f = if class(0, 0x31, 0x19e) {
        kf(0x354)
    } else if class(1, 0x19d, 0x21f) {
        kf(0x358)
    } else if class(2, 0x274, 0x2ac) {
        kf(0x360)
    } else {
        kf(0x35c)
    };
    f.wrapping_mul(sum) as i64
}
        };
        assert_eq!(got, want_eax, "sum_rules_cost — eax, row `{line}`");
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "sum_rules_cost — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "sum_rules_cost — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[2 + i], "sum_rules_cost — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("Type::sum_rules_cost@006680e0: {n} rows agree");
}
