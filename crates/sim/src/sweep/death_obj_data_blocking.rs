//! `DeathObjData::blocking@008d4a90` under the emulator against the port (item 1976; hand-written on tools/emu/hooks.py).

/// `DeathObjData::blocking@008d4a90`, run by `tools/emu/sweep_death_obj_data_blocking.py` under unicorn
/// (tools/emu): `units[who][o]`'s slot 0x120 answer, nonzero → 1; the call's `ecx` is the type record at the object's `+0x18`.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_death_obj_data_blocking.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (lhs, rhs) = line.split_once(" -> ").expect("`<args…> -> <result>`");
        let a: Vec<i64> = lhs
            .split_whitespace()
            .map(|s| s.parse().expect("an integer"))
            .collect();
        let rhs = rhs.trim();
        let (eax, seen) = rhs.split_once(" | ").expect("`<eax> | <ecx>`");
        assert_eq!(
            eax.trim().parse::<i64>().unwrap(),
            i64::from(a[2] != 0),
            "blocking — answer, row `{line}`"
        );
        assert_eq!(
            seen.trim().parse::<i64>().unwrap(),
            a[3],
            "blocking — the call, row `{line}`"
        );
        n += 1;
    }
    assert!(n >= 250, "only {n} rows");
    eprintln!("DeathObjData::blocking@008d4a90: {n} rows agree");
}
