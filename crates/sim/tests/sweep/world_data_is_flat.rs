//! `WorldData::is_flat@006b3120` under the emulator against the port (item 2044; hand-written on tools/emu/hooks.py).

/// `WorldData::is_flat@006b3120`, run by `tools/emu/sweep_world_data_is_flat.py` under unicorn (tools/emu): a
/// cell flagged `world::cell::MOUNTAIN` or `FOREST` is not flat and the sea is not asked; else the sea's
/// answer is asked of the world with the same two pointers, and a sea cell, a `ROCK` one or one with the
/// unnamed `0x40` is not flat. The census's `port` mark is `orders.rs`'s unrelated building-type `is_flat`.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    use crate::world::cell::{FOREST, MOUNTAIN, ROCK};
    let Some(rows) = super::emu_rows("sweep_world_data_is_flat.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (lhs, rhs) = line
            .split_once(" -> ")
            .expect("`<args…> -> <eax> | <calls>`");
        let (eax, calls) = rhs.split_once(" | ").unwrap_or((rhs, ""));
        let a: Vec<i64> = lhs
            .split_whitespace()
            .map(|s| s.parse().expect("an integer"))
            .collect();
        let (flags, sea) = (a[3] as u16, a[4] != 0);
        let asked = flags & (MOUNTAIN | FOREST) == 0;
        let flat = asked && !sea && flags & (ROCK | 0x40) == 0;
        assert_eq!(
            calls.trim(),
            if asked { "1,1,1" } else { "" },
            "is_flat — the call, row `{line}`"
        );
        assert_eq!(
            eax.trim().parse::<i64>().expect("an integer"),
            i64::from(flat),
            "is_flat — row `{line}`"
        );
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("WorldData::is_flat@006b3120: {n} rows agree");
}
