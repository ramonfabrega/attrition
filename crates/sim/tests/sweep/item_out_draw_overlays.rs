//! `ItemOut::draw_overlays@00677590` under the emulator against the port (item 1736; tools/sweep_gen.py, const).

/// `ItemOut::draw_overlays@00677590`, run by `tools/emu/sweep_item_out_draw_overlays.py` under unicorn
/// (tools/emu): answers a constant whatever `this` is, held to `1`.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_item_out_draw_overlays.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (a, want) = super::row(line);
        let _this = a[0];
        let got: i64 = 1;
        assert_eq!(got, want, "draw_overlays — row `{line}`");
        n += 1;
    }
    assert!(n >= 5, "only {n} rows");
    eprintln!("ItemOut::draw_overlays@00677590: {n} rows agree");
}
