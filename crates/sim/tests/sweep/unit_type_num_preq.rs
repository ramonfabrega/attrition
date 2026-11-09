//! `UnitTypeData::num_preq@00470810` under the emulator against the port (item 1705).

use crate::flag_reads::NUM_PREQ;

/// `UnitTypeData::num_preq@00470810`, run by `tools/emu/sweep_unit_type_num_preq.py` under unicorn
/// (tools/emu): returns 2 whatever the record holds — the rows fill the record with seeded words — against `flag_reads::NUM_PREQ`, the count of prerequisites a unit type is built from (the tree loader's two slots).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_unit_type_num_preq.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (_, want) = super::row(line);
        assert_eq!(want, NUM_PREQ, "num_preq — row `{line}`");
        n += 1;
    }
    assert!(n >= 60, "only {n} rows");
    eprintln!("UnitTypeData::num_preq@00470810: {n} rows agree");
}
