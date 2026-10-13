//! `TypeData::is_age_type@00470820` under the emulator against the port (item 1799; tools/sweep_gen.py, range).

/// `TypeData::is_age_type@00470820`, run by `tools/emu/sweep_type_data_is_age_type.py` under unicorn
/// (tools/emu): every index -2..=0x33f and the edges, against `(0x220..=0x226).contains(&t)`.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_type_data_is_age_type.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (a, want) = super::row(line);
        let t = i32::try_from(a[0]).expect("an int");
        let got: bool = (0x220..=0x226).contains(&t);
        assert_eq!(got, want != 0, "is_age_type — row `{line}`");
        n += 1;
    }
    assert!(n >= 800, "only {n} rows");
    eprintln!("TypeData::is_age_type@00470820: {n} rows agree");
}
