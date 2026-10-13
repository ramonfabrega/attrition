//! `UnitData::get_combat_stance@00610a30` under the emulator against the port (item 2072; tools/sweep_gen.py, vcall --base).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

fn unhex(s: &str) -> Vec<u8> {
    (0..s.len() / 2)
        .map(|i| u8::from_str_radix(&s[2 * i..2 * i + 2], 16).expect("hex"))
        .collect()
}

/// `UnitData::get_combat_stance@00610a30`, run by `tools/emu/sweep_unit_data_get_combat_stance.py` under unicorn
/// (tools/emu): slot +0x104 against the base 0x0046ced0, a row each arm, the slots' answer over the signed-32 edges and random words, 512 random bytes of object; the calls and `eax` held.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_unit_data_get_combat_stance.py") else {
        return;
    };
    let handed: &[i64] = &[0];
    let args: String = handed.iter().map(|h| format!(",{h}")).collect();
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (lhs, rhs) = line
            .split_once(" -> ")
            .expect("a row is `<mode> <v> | <object> -> <eax> | <calls>`");
        let (head, obj) = lhs.split_once(" | ").expect("`<mode> <v> | <object>`");
        let (eax, calls) = rhs.split_once(" | ").expect("`<eax> | <calls>`");
        let a: Vec<i64> = head
            .split_whitespace()
            .map(|s| s.parse().expect("an integer"))
            .collect();
        let (mode, v) = (a[0], i32::try_from(a[1]).expect("a 32-bit answer"));
        let m = unhex(obj.trim());
        let want_calls = match (mode, true) {
            (0, _) => format!("this{args}"),
            (_, true) => format!("via{args}"),
            _ => String::new(),
        };
        assert_eq!(
            calls.trim(),
            want_calls,
            "get_combat_stance — calls, row `{line}`"
        );
        let got: i64 = {
            let (mode, v, m) = (mode, v, &m[..]);
            let _ = (mode, v, m);
            i64::from(crate::stance::get_stance(
                v != 0,
                m[0xb1],
                crate::stance::StanceType::Combat,
            ))
        };
        assert_eq!(
            got,
            eax.trim().parse::<i64>().expect("an integer"),
            "get_combat_stance — eax, row `{line}`"
        );
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("UnitData::get_combat_stance@00610a30: {n} rows agree");
}
