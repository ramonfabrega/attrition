//! `WallData::is_under_attack@00472420` under the emulator against the port (item 1701).

/// `WallData::is_under_attack@00472420`, run by `tools/emu/sweep_wall_is_under_attack.py` under unicorn
/// (tools/emu): `build_masks` (short at `+0x60`) `& 0x20`, over every bit edge and a seeded band, against `Building::is_under_attack` — the port keeps the two-stage latch (`build_masks & 0x10 | 0x20`) as `under_attack`'s bits 0x1 and 0x2, so the row sets bit 1 from bit 5 of the mask.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_wall_is_under_attack.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (a, want) = super::row(line);
        let mask = i16::try_from(a[0]).expect("a 16-bit mask");
        let (mut s, _) = super::sim_with_unit(crate::UnitType::default());
        let b = s.add_building(0, crate::world::Pos::new(0x1000, 0x1000), 0);
        s.buildings[b].under_attack =
            u8::from(mask & 0x10 != 0) | (u8::from(mask & 0x20 != 0) << 1);
        assert_eq!(
            s.buildings[b].is_under_attack(),
            want != 0,
            "is_under_attack — row `{line}`"
        );
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("WallData::is_under_attack@00472420: {n} rows agree");
}
