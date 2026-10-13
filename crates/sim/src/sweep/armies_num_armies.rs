//! `Armies::num_armies@006f3200` under the emulator against the port (item 2038; hand-written on tools/emu/hooks.py).

/// `Armies::num_armies@006f3200`, run by `tools/emu/sweep_armies_num_armies.py` under unicorn (tools/emu): of
/// the player's sixteen army slots, those with a nonzero `valid` short, a status sharing a bit with the mask, and
/// — unless the region argument is negative — that region, are counted; `this` is never read. `army.rs`'s
/// `num_armies` is the port, its `Option<u16>` the negative argument's "any region".
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_armies_num_armies.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (a, want) = super::row(line);
        let (mask, reg) = (a[1] as i32, a[2] as i32);
        let got = a[3..]
            .chunks(3)
            .filter(|s| s[0] != 0 && s[1] as i32 & mask != 0 && (reg < 0 || s[2] as i32 == reg))
            .count() as i64;
        assert_eq!(got, want, "num_armies — row `{line}`");
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("Armies::num_armies@006f3200: {n} rows agree");
}
