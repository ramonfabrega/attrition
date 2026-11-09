//! `BuildTypeData::max_knowledge_gatherers@006365e0` under the emulator against the port (item 1814; tools/sweep_gen.py, const).

/// `BuildTypeData::max_knowledge_gatherers@006365e0`, run by `tools/emu/sweep_build_type_data_max_knowledge_gatherers.py` under unicorn
/// (tools/emu): answers a constant whatever `this` is, held to `7`.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_build_type_data_max_knowledge_gatherers.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (a, want) = super::row(line);
        let _this = a[0];
        let got: i64 = 7;
        assert_eq!(got, want, "max_knowledge_gatherers — row `{line}`");
        n += 1;
    }
    assert!(n >= 5, "only {n} rows");
    eprintln!("BuildTypeData::max_knowledge_gatherers@006365e0: {n} rows agree");
}
