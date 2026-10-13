//! `WallData::get_garrison_limit@0063ed40` under the emulator against the port (item 1966; hand-written on tools/emu/hooks.py).

fn unhex(s: &str) -> Vec<u8> {
    (0..s.len() / 2)
        .map(|i| u8::from_str_radix(&s[2 * i..2 * i + 2], 16).expect("hex"))
        .collect()
}

/// `WallData::get_garrison_limit@0063ed40`, run by `tools/emu/sweep_wall_data_get_garrison_limit.py` under unicorn
/// (tools/emu): the callee hooked; `ecx` = the word at `this+0x18`, the argument the byte at `this+9`, `eax` the callee's answer.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_wall_data_get_garrison_limit.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (lhs, rhs) = line.split_once(" -> ").expect("`<ans> | <hex> -> <eax> | <calls>`");
        let (ans, hex) = lhs.split_once(" | ").expect("`<ans> | <hex>`");
        let (eax, seen) = rhs.split_once(" | ").expect("`<eax> | <calls>`");
        let m = unhex(hex.trim());
        let ecx = u32::from_le_bytes(m[0x18..0x1c].try_into().expect("four bytes"));
        let want = format!("{ecx},{}", m[9]);
        assert_eq!(seen.trim(), want, "get_garrison_limit — call, row `{line}`");
        let (a, e): (i64, i64) = (ans.trim().parse().unwrap(), eax.trim().parse().unwrap());
        assert_eq!(e, a, "get_garrison_limit — eax, row `{line}`");
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("WallData::get_garrison_limit@0063ed40: {n} rows agree");
}
