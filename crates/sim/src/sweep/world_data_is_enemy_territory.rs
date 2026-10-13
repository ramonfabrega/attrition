//! `WorldData::is_enemy_territory@006b2490` under the emulator against the port (item 2112; hand-written on tools/emu/hooks.py).

use crate::territory::enemy_territory_by;

/// `WorldData::is_enemy_territory@006b2490`, run by `tools/emu/sweep_world_data_is_enemy_territory.py` under
/// unicorn (tools/emu): the cell's owner byte through the world's width and record table, `who`, the leader's own
/// `who` and the two `diplos` words, drawn per row with the cell and the leaders placed where the reads land. The
/// port's [`enemy_territory_by`] answers the two `diplos` reads from the row and refuses any other, so which
/// leader's word is read, toward whom, is held, and the answer to `eax`.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_world_data_is_enemy_territory.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (a, want) = super::row(line);
        let (owner, who, me, d1, d2) = (
            a[3] as i32,
            a[4] as i32,
            a[5] as i32,
            a[6] as i32,
            a[7] as i32,
        );
        let got = enemy_territory_by(owner, who, me, |of, toward| match (of, toward) {
            (o, t) if o == who && t == owner => d1,
            (o, t) if o == owner && t == me => d2,
            _ => panic!("is_enemy_territory — read diplos[{of}][{toward}], row `{line}`"),
        });
        assert_eq!(
            i64::from(got),
            want,
            "is_enemy_territory — eax, row `{line}`"
        );
        n += 1;
    }
    assert!(n >= 400, "only {n} rows");
    eprintln!("WorldData::is_enemy_territory@006b2490: {n} rows agree");
}
