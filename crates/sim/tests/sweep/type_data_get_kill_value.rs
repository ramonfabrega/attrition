//! `TypeData::get_kill_value@00663a00` under the emulator against the port (item 1954; tools/sweep_gen.py, vcall).

/// `TypeData::get_kill_value@00663a00`, run by `tools/emu/sweep_type_data_get_kill_value.py` under unicorn
/// (tools/emu): the slot's answer over the signed-32 edges and two hundred random words, against `i64::from(v)`.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_type_data_get_kill_value.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (line, handed) = line
            .split_once("  # slot handed ")
            .expect("the slot's argument");
        assert_eq!(
            handed.trim(),
            "0",
            "get_kill_value hands the slot 0 — row `{line}`"
        );
        let (a, want) = super::row(line);
        let v = i32::try_from(a[0]).expect("a 32-bit integer");
        let got: i64 = i64::from(v);
        assert_eq!(got, want, "get_kill_value — row `{line}`");
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("TypeData::get_kill_value@00663a00: {n} rows agree");
}
