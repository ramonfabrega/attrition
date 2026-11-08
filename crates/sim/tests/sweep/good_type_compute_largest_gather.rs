//! `GoodType::compute_largest_gather@0066e920` against `world::largest_gather` (item 1619).

/// `GoodType::compute_largest_gather@0066e920`, run under unicorn by
/// `tools/emu/sweep_good_type_compute_largest_gather.py` with the `lands`
/// array (global `0xe3a380`) laid out by hand from `world::LANDS`: every row
/// `<good> <noise>` is the good's type index and a seed that fills the two
/// lands the original skips (1 and 2) and every byte outside the eight
/// cells with junk, and the answer is the word written at `this+0x2ec`.
/// Asserted against `crate::world::largest_gather`. The port is hardwired to
/// the shipped table, so a table that differs in a land the original reads
/// cannot be a row (and on the shipped table every answer is 1: the
/// `max` and the clamp's upper bound are exercised by the original only in
/// the script's own probes, not in a row the port can answer). The port
/// takes a `usize`; a negative good (`-1` is TYPE_NONE, what the empty cells
/// hold) is asked as `usize::MAX`, the port's "no land makes it".
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_good_type_compute_largest_gather.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (args, want) = super::row(line);
        let good = usize::try_from(args[0]).unwrap_or(usize::MAX);
        let ours = i64::from(crate::world::largest_gather(good));
        assert_eq!(ours, want, "largest_gather({}) — row `{line}`", args[0]);
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("GoodType::compute_largest_gather@0066e920: {n} rows agree");
}
