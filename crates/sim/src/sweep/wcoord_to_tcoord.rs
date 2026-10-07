/// `WCoord::operator_TCoord@004613b0` — `*out = value * 4 + 2`, a cell
/// coordinate's centre tile — against `Cell::centre_tile` (one axis of it),
/// over the rows `tools/emu/sweep_wcoord_to_tcoord.py` prints from the
/// original's own machine code: edges around zero and the cell/tile scales,
/// then a seeded random set, all inside the domain where `value * 4 + 2`
/// fits an `i32`.
#[test]
fn wcoord_to_tcoord_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_wcoord_to_tcoord.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines() {
        let (args, want) = super::row(line);
        let v = i32::try_from(args[0]).expect("a WCoord value is an int");
        let got = crate::world::Cell::new(v, 0).centre_tile().x;
        assert_eq!(i64::from(got), want, "row `{line}`: ours {got}");
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
}
