//! `BuildTypeData::get_city_level@00639800` under the emulator against the port (item 1971; hand-written on tools/emu/hooks.py).

/// `BuildTypeData::get_city_level@00639800`, run by `tools/emu/sweep_build_type_data_get_city_level.py` under unicorn
/// (tools/emu): the type index `+0x4` against `0x19f`, `0x1a0` and `0x213`, every index -2..=0x33f and the edges.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_build_type_data_get_city_level.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (a, want) = super::row(line);
        let got: i64 = match a[0] as i32 {
            0x19f => 2,
            0x1a0 | 0x213 => 3,
            _ => 1,
        };
        assert_eq!(got, want, "get_city_level — row `{line}`");
        n += 1;
    }
    assert!(n >= 800, "only {n} rows");
    eprintln!("BuildTypeData::get_city_level@00639800: {n} rows agree");
}
