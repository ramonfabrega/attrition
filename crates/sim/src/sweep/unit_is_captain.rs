//! `UnitData::is_captain@0046ceb0` under the emulator against the port (item 1691).

/// `UnitData::is_captain@0046ceb0`, run by `tools/emu/sweep_unit_is_captain.py` under unicorn
/// (tools/emu): `o_up < 0`, `o_up` the signed short at `unit+0x8e`, over the short's edges and a seeded band, against `Sim::is_captain`. The port keeps the squad head as `Unit::captain`, a bool, where the original keeps `o_up == -1`; the row sets the port's bool from the sign of `o_up`, so what is proved is the sign test and that nothing else (location, garrison) enters.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_unit_is_captain.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (a, want) = super::row(line);
        let (mut s, u) = super::sim_with_unit(crate::UnitType::default());
        s.units[u].captain = a[0] < 0;
        s.units[u].on_map = a[0] % 2 == 0;
        assert_eq!(s.is_captain(u), want != 0, "is_captain — row `{line}`");
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("UnitData::is_captain@0046ceb0: {n} rows agree");
}
