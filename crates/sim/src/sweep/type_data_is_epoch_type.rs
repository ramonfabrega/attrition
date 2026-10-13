//! `TypeData::is_epoch_type@00470870` under the emulator against the port (item 1799; tools/sweep_gen.py, range).

/// `TypeData::is_epoch_type@00470870`, run by `tools/emu/sweep_type_data_is_epoch_type.py` under unicorn
/// (tools/emu): every index -2..=0x33f and the edges, against `(0x227..=0x242).contains(&t)`.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_type_data_is_epoch_type.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (a, want) = super::row(line);
        let t = i32::try_from(a[0]).expect("an int");
        let got: bool = (0x227..=0x242).contains(&t);
        assert_eq!(got, want != 0, "is_epoch_type — row `{line}`");
        n += 1;
    }
    assert!(n >= 800, "only {n} rows");
    eprintln!("TypeData::is_epoch_type@00470870: {n} rows agree");
}
