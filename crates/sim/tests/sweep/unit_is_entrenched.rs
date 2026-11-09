//! `UnitData::is_entrenched@0060a810` under the emulator against the port (item 1704).

use crate::type_index::is_entrenched;

/// `UnitData::is_entrenched@0060a810`, run by `tools/emu/sweep_unit_is_entrenched.py` under unicorn
/// (tools/emu): `unit_masks & 0x2000000`, and with a non-zero argument only without `unit_masks2 & 0x20000`, over the cross of the bit edges and a seeded band, against `type_index::is_entrenched` — the reader `find_nearby_target`'s `local_24` and the combat step's entrenchment gate stand on.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_unit_is_entrenched.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (a, want) = super::row(line);
        let (masks, masks2) = (a[0] as u32, a[1] as u32);
        let param = i32::try_from(a[2]).expect("an int");
        assert_eq!(
            is_entrenched(masks, masks2, param),
            want != 0,
            "is_entrenched — row `{line}`"
        );
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("UnitData::is_entrenched@0060a810: {n} rows agree");
}
