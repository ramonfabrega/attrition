//! `ObjectTypeData::is_defense@00470470` under the emulator against the port (item 1993; hand-written on tools/emu/hooks.py).

/// `ObjectTypeData::is_defense@00470470`, run by `tools/emu/sweep_object_type_data_is_defense.py` under unicorn
/// (tools/emu): slot `0xfc` then (when zero) slot `0x100`, each on the same `this`; any non-zero → 1.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_object_type_data_is_defense.py") else {
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
        let (eax, seen) = rhs.split_once(" | ").expect("`<eax> | <calls>`");
        let (this, fc, s100) = (a[0], a[1], a[2]);
        let want = if fc != 0 {
            format!("slotfc:{this}")
        } else {
            format!("slotfc:{this};slot100:{this}")
        };
        assert_eq!(seen.trim(), want, "is_defense — the calls, row `{line}`");
        assert_eq!(
            eax.trim().parse::<i64>().unwrap(),
            i64::from(fc != 0 || s100 != 0),
            "is_defense — answer, row `{line}`"
        );
        n += 1;
    }
    assert!(n >= 250, "only {n} rows");
    eprintln!("ObjectTypeData::is_defense@00470470: {n} rows agree");
}
