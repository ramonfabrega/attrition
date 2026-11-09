//! `UnitTypeData::research_time@0061d760` under the emulator against the port (item 1998; hand-written on tools/emu/hooks.py).

/// `UnitTypeData::research_time@0061d760`, run by `tools/emu/sweep_unit_type_data_research_time.py` under unicorn
/// (tools/emu): the base type's `research_time` (hooked — not run, so not backed here — handed the argument and `this`) times `research_premium_time`
/// (`this+0x2e4`), a wrapping 32-bit product divided by 256 toward zero. [`crate::production::base_time`] writes
/// the step as `>> 8`, which floors: the two agree on every non-negative product, and the shipped values give
/// nothing else (both factors are positive); the rows reach negative products too, held to the listing.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_unit_type_data_research_time.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (a, want) = super::row(line);
        let (p0, t, prem) = (a[0], a[1] as i32, a[2] as i32);
        assert_eq!(
            (a[3], a[4]),
            (p0, 1),
            "the callee's argument and ecx — row `{line}`"
        );
        let product = t.wrapping_mul(prem);
        let got = i64::from(product / 256);
        if product >= 0 {
            assert_eq!(
                i64::from(product >> 8),
                got,
                "the port's shift — row `{line}`"
            );
        }
        assert_eq!(got, want, "research_time — row `{line}`");
        n += 1;
    }
    assert!(n >= 400, "only {n} rows");
    eprintln!("UnitTypeData::research_time@0061d760: {n} rows agree");
}
