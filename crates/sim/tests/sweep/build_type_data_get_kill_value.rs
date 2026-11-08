//! `BuildTypeData::get_kill_value@00639790` under the emulator against the port (item 2112; hand-written on tools/emu/hooks.py).

use crate::cost::kill_value_by;

/// `BuildTypeData::get_kill_value@00639790`, run by `tools/emu/sweep_build_type_data_get_kill_value.py` under
/// unicorn (tools/emu): the base slot `+0x7c` asked first with `0`, then, for `who >= 0`, the two support words
/// times the factor times the leader's built and ordered counts of the type, wrapping. The port's
/// [`kill_value_by`] is handed the row's words; the one call is held to the slot on `this`, and the value to
/// `eax`.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_build_type_data_get_kill_value.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (a, want, tail) = super::tail_row(line);
        assert_eq!(
            tail[0], "base,this,0",
            "get_kill_value — the call, row `{line}`"
        );
        let who = a[0] as i32;
        let counts = (who >= 0).then(|| (a[5] as u16, a[6] as u16));
        let got = kill_value_by(a[7] as i32, (a[2] as i32, a[3] as i32), a[4] as i32, counts);
        assert_eq!(i64::from(got), want, "get_kill_value — eax, row `{line}`");
        n += 1;
    }
    assert!(n >= 400, "only {n} rows");
    eprintln!("BuildTypeData::get_kill_value@00639790: {n} rows agree");
}
