//! `SupplyData::get_radius@0073b560` under the emulator against the port (item 2112; hand-written on tools/emu/hooks.py).

use crate::Tuning;
use crate::supply::supply_radius;

/// `SupplyData::get_radius@0073b560`, run by `tools/emu/sweep_supply_data_get_radius.py` under unicorn
/// (tools/emu): `supply_radius + supply_radius_upgrade · get_supply_upgrade`, plus `terra_cotta_range` when the
/// leader holds wonder `0x211`, the three constants and both answers drawn per row. The port's
/// [`supply_radius`] is handed a [`Tuning`] carrying the row's constants; the two asks are held to the leader
/// `leaders + 0x6eec · who`, the upgrade first.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_supply_data_get_radius.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (a, want, tail) = super::tail_row(line);
        let who = a[3] as i32;
        let ecx = (a[4] as u32).wrapping_add(who.wrapping_mul(0x6eec) as u32);
        assert_eq!(
            tail[0],
            format!("upgrade,{ecx};wonder,{ecx},{}", 0x211),
            "get_radius — the asks, row `{line}`"
        );
        let t = Tuning {
            supply_radius: a[0] as i32,
            supply_radius_upgrade: a[1] as i32,
            terra_cotta_range: a[2] as i32,
            ..Tuning::RON
        };
        let got = supply_radius(&t, a[5] as i32, a[6] != 0);
        assert_eq!(i64::from(got), want, "get_radius — eax, row `{line}`");
        n += 1;
    }
    assert!(n >= 400, "only {n} rows");
    eprintln!("SupplyData::get_radius@0073b560: {n} rows agree");
}
