//! `WallData::construct_time@00642d90` under the emulator against the port (item 1701).

/// `WallData::construct_time@00642d90`, run by `tools/emu/sweep_wall_construct_time.py` under unicorn
/// (tools/emu): the `constr_time` word (ulong at `+0x50`, the argument ignored) over 32-bit edges and a seeded band, against `Building::constr_time` — the stored base `build::construct_base` writes. This is `WallData`'s raw read; `build::construct_time` is `BuildData`'s override (the clock mods), not this function.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_wall_construct_time.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (a, want) = super::row(line);
        let v = i32::try_from(i64::from(u32::try_from(a[0]).expect("a ulong")) as i32 as i64)
            .expect("a 32-bit word");
        let (mut s, _) = super::sim_with_unit(crate::UnitType::default());
        let b = s.add_building(0, crate::world::Pos::new(0x1000, 0x1000), 0);
        s.buildings[b].constr_time = v;
        assert_eq!(
            i64::from(s.buildings[b].constr_time as u32),
            want,
            "construct_time — row `{line}`"
        );
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("WallData::construct_time@00642d90: {n} rows agree");
}
