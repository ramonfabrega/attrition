//! `ObjectData::attack_dist@0064c880` under the emulator against the port (item 1966; hand-written on tools/emu/hooks.py).

fn unhex(s: &str) -> Vec<u8> {
    (0..s.len() / 2)
        .map(|i| u8::from_str_radix(&s[2 * i..2 * i + 2], 16).expect("hex"))
        .collect()
}

/// `ObjectData::attack_dist@0064c880`, run by `tools/emu/sweep_object_data_attack_dist.py` under unicorn
/// (tools/emu): the five-argument callee hooked; its arguments are `p0`, `p1` and the two encoded words XOR 0x63637.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_object_data_attack_dist.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (lhs, rhs) = line.split_once(" -> ").expect("`<ans> <p0> <p1> | <hex> -> <eax> | <calls>`");
        let (args, hex) = lhs.split_once(" | ").expect("`<args> | <hex>`");
        let (eax, seen) = rhs.split_once(" | ").expect("`<eax> | <calls>`");
        let a: Vec<i64> = args.split_whitespace().map(|s| s.parse().unwrap()).collect();
        let m = unhex(hex.trim());
        let w = |o: usize| i32::from_le_bytes(m[o..o + 4].try_into().expect("four bytes"));
        // `this` is the harness's heap address: the first call's `ecx`
        let this: i64 = seen.split(';').next().unwrap().split(',').next().unwrap().parse().unwrap();
        let want = format!("{this},{},{},{},{}", a[1], a[2], w(0x10) ^ 0x63637, w(0x14) ^ 0x63637);
        assert_eq!(seen.trim(), want, "attack_dist — call, row `{line}`");
        assert_eq!(eax.trim().parse::<i64>().unwrap(), i64::from(a[0] as i32), "attack_dist — eax, row `{line}`");
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("ObjectData::attack_dist@0064c880: {n} rows agree");
}
