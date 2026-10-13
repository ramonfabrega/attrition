//! `WallData::has_repeat_air@00472410` under the emulator against the port (item 1701).

/// `WallData::has_repeat_air@00472410`, run by `tools/emu/sweep_wall_has_repeat_air.py` under unicorn
/// (tools/emu): `build_masks` (short at `+0x60`) `& 0x80`, over every bit edge and a seeded band, against `Building::repeat_air` — the port's bool for that bit, which the row sets from the mask; what is proved is the bit's identity (0x80), that the sign bit does not leak, and that nothing else enters.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_wall_has_repeat_air.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (a, want) = super::row(line);
        let mask = i16::try_from(a[0]).expect("a 16-bit mask");
        let (mut s, _) = super::sim_with_unit(crate::UnitType::default());
        let b = s.add_building(0, crate::world::Pos::new(0x1000, 0x1000), 0);
        s.buildings[b].repeat_air = mask & 0x80 != 0;
        assert_eq!(
            s.buildings[b].repeat_air,
            want != 0,
            "has_repeat_air — row `{line}`"
        );
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("WallData::has_repeat_air@00472410: {n} rows agree");
}
