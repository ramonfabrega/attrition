//! `WorldData::is_built_at@0046f880` under the emulator against the port (item 1630).

use crate::world::is_built_at;

/// `WorldData::is_built_at@0046f880`, run by `tools/emu/sweep_world_is_built_at.py`
/// under the unicorn emulator (tools/emu): `ecx` the world, two pointers to the
/// `TCoord` words, over a `TData` array (`this+0x138`, stride 2) laid out in
/// mapped memory, with every other tile a decoy so a read of the wrong tile
/// shows. The port is the tile-mask predicate `world::is_built_at`, which takes
/// the mask and not the world, so it is driven with the queried tile's mask
/// and asserted against the original's answer. Covers every low byte over four
/// high bytes, every single bit and its complement, and random masks.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_world_is_built_at.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (a, want) = super::row(line);
        let mask = u16::try_from(a[4]).expect("a 16-bit mask");
        assert_eq!(
            i64::from(is_built_at(mask)),
            want,
            "is_built_at — row `{line}`"
        );
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("is_built_at@0046f880: {n} rows agree");
}
