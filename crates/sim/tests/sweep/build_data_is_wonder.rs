//! `BuildData::is_wonder@00472320` under the emulator against the port (item 2090; hand-written on tools/emu/hooks.py).

use crate::type_index::is_wonder_type;

/// `BuildData::is_wonder@00472320`, run by `tools/emu/sweep_build_data_is_wonder.py` under unicorn (tools/emu):
/// the type record at `+0x18`; with `BuildType::vftable` its type index against [`is_wonder_type`]
/// (`0x20e..=0x21e`) and no call, else a jump to the record's own slot `+0x1c`, whose answer passes through
/// unchanged, asked once on the record.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_build_data_is_wonder.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (a, want, tail) = super::tail_row(line);
        let (real, t, ans) = (a[0] != 0, i32::try_from(a[1]).expect("a type"), a[2]);
        let (got, calls) = if real {
            (i64::from(is_wonder_type(t)), "")
        } else {
            (ans, "rec")
        };
        assert_eq!(tail[0], calls, "is_wonder — the calls, row `{line}`");
        assert_eq!(got, want, "is_wonder — eax, row `{line}`");
        n += 1;
    }
    assert!(n >= 400, "only {n} rows");
    eprintln!("BuildData::is_wonder@00472320: {n} rows agree");
}
