//! `Armies::find_city@006f3160` under the emulator against the port (item 2044; hand-written on tools/emu/hooks.py).

/// `Armies::find_city@006f3160`, run by `tools/emu/sweep_armies_find_city.py` under unicorn (tools/emu): the
/// first of the player's sixteen army slots with a nonzero `valid` short, the city as its target and the
/// mustering bit in its status (`army.rs`'s `status::MUSTERING`), else -1; `this` and the third argument are
/// never read. The census's `port` mark is `city.rs`'s unrelated `find_city`; `army.rs` keeps the fields
/// (`valid`, `target`, `status`) this walk reads.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    use crate::army::status::MUSTERING;
    let Some(rows) = super::emu_rows("sweep_armies_find_city.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (a, want) = super::row(line);
        let city = a[1] as i32;
        let got = a[3..]
            .chunks(3)
            .position(|s| s[0] != 0 && s[2] as i32 == city && s[1] as i32 & MUSTERING != 0)
            .map_or(-1, |k| k as i64);
        assert_eq!(got, want, "find_city — row `{line}`");
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("Armies::find_city@006f3160: {n} rows agree");
}
