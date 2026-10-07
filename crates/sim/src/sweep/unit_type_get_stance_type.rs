//! `UnitTypeData::get_stance_type@0061d350` under the emulator against
//! `stance::stance_type` (item 1619).

use crate::stance::{StanceType, stance_type};

/// `UnitTypeData::get_stance_type@0061d350`, run by
/// `tools/emu/sweep_unit_type_get_stance_type.py` under unicorn: `role`
/// (`type+0x2c8`), `unit_flags2` (`type+0x2b8`) and the type index (`type+4`)
/// set per row, answers 0 combat, 1 worker, 2 caster, 3 packer, -1 none.
/// Every row — all 256 low bytes of the flags against military and civilian
/// roles and the index boundaries 0x31..=0x36, plus a seeded random band — is
/// asserted against `crate::stance::stance_type`, which is the whole function
/// (no state beyond the three words).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_unit_type_get_stance_type.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (args, want) = super::row(line);
        let role = u32::try_from(args[0]).expect("a 32-bit role");
        let flags2 = u32::try_from(args[1]).expect("32-bit flags");
        let idx = i32::try_from(args[2]).expect("a 32-bit index");
        let ours = match stance_type(role, flags2, idx) {
            StanceType::Combat => 0,
            StanceType::Worker => 1,
            StanceType::Caster => 2,
            StanceType::Packer => 3,
            StanceType::None => -1,
        };
        assert_eq!(
            ours, want,
            "stance_type({role:#x}, {flags2:#x}, {idx:#x}) — row `{line}`"
        );
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("get_stance_type@0061d350: {n} rows agree");
}
