//! `ObjectData::health_level@006534b0` under the emulator against the port (item 2061; hand-written on
//! tools/emu/hooks.py).

/// `ObjectData::health_level@006534b0`, run by `tools/emu/sweep_object_data_health_level.py` under unicorn
/// (tools/emu), held to `city.rs`'s `health_level(hits, damage)`: the hits are `this`'s slot `+0x11c` asked
/// with 0; the damage is `this+0x24`, or — when its slot `+0x18` says it is a unit's figure — the unit's total
/// damage, asked of the unit the per-player table names by `this`'s `who` byte and its short `+0xa`, with
/// `(0, 0)`. The rows draw hits non-negative (a negative pool the original clamps to zero and the port does
/// not reach) and damage at every threshold's edge.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_object_data_health_level.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (a, eax, tail) = super::tail_row(line);
        let (hits, damage, figure) = (a[0] as i32, a[1] as i32, a[2] != 0);
        let calls = if figure {
            "hits:1,figure:1,total:1"
        } else {
            "hits:1,figure:1"
        };
        assert_eq!(tail, [calls], "health_level — the calls, row `{line}`");
        assert_eq!(
            eax,
            i64::from(crate::city::health_level(hits, damage)),
            "health_level — row `{line}`"
        );
        n += 1;
    }
    assert!(n >= 300, "only {n} rows");
    eprintln!("ObjectData::health_level@006534b0: {n} rows agree");
}
