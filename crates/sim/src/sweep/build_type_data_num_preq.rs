//! `BuildTypeData::num_preq@00472a60` under the emulator against the port (item 1735).

/// `BuildTypeData::num_preq@00472a60`, run by `tools/emu/sweep_build_type_data_num_preq.py` under unicorn
/// (tools/emu): returns 3 whatever the record holds, against `flag_reads::BUILD_NUM_PREQ` (the count `tech::Tech::num_preq` returns for a building or a tech).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_build_type_data_num_preq.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (_, want) = super::row(line);
        assert_eq!(
            want,
            crate::flag_reads::BUILD_NUM_PREQ,
            "BuildTypeData::num_preq — row `{line}`"
        );
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("BuildTypeData::num_preq@00472a60: {n} rows agree");
}
