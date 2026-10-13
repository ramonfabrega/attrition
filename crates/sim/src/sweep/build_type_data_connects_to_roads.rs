//! `BuildTypeData::connects_to_roads@006398c0` under the emulator against the port (item 2038; hand-written on tools/emu/hooks.py).

use crate::build::flags;

/// `BuildTypeData::connects_to_roads@006398c0`, run by `tools/emu/sweep_build_type_data_connects_to_roads.py`
/// under unicorn (tools/emu): a gatherer ([`flags::GATHER`], read inline when its own slot `+0x90` is the base,
/// asked of the slot otherwise) or a type needing no city ([`flags::NO_CITY`]) connects only if slot `+0x60`
/// says it is `0x1a4`, the University; anything else connects. `roads.rs`'s `connects_to_roads` is the port.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_build_type_data_connects_to_roads.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (a, want) = super::row(line);
        let (mode, f, g, is) = (a[0], a[1] as i32 as u32, a[2], a[3]);
        assert_eq!(a[7], 1, "every slot's ecx is this — row `{line}`");
        let gather = if mode == 1 {
            f & flags::GATHER != 0
        } else {
            g != 0
        };
        let asks = gather || f & flags::NO_CITY != 0;
        let calls = match (mode == 1, asks) {
            (true, false) => 0,
            (true, true) => 2,
            (false, false) => 1,
            (false, true) => 12,
        };
        assert_eq!(a[4], calls, "the slots asked — row `{line}`");
        if asks {
            assert_eq!((a[5], a[6]), (0x1a4, 0), "is(University, 0) — row `{line}`");
        }
        let got = !asks || is != 0;
        assert_eq!(i64::from(got), want, "connects_to_roads — row `{line}`");
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("BuildTypeData::connects_to_roads@006398c0: {n} rows agree");
}
