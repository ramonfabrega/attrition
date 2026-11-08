//! `GraphicWinXML::get_graphicwinxml@0041c000` under the emulator against the port (item 1782).

/// `GraphicWinXML::get_graphicwinxml@0041c000`, run by `tools/emu/sweep_graphicwinxml_get_graphicwinxml.py` under unicorn
/// (tools/emu): `this` as it stands, against `flag_reads::graphic_win_xml`.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_graphicwinxml_get_graphicwinxml.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (a, want) = super::row(line);
        let t = i32::try_from(a[0]).expect("a 32-bit address");
        assert_eq!(
            want,
            i64::from(crate::flag_reads::graphic_win_xml(t)),
            "GraphicWinXML::get_graphicwinxml — row `{line}`"
        );
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("GraphicWinXML::get_graphicwinxml@0041c000: {n} rows agree");
}
