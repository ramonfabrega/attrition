//! `Group::report_speed@00713bb0` under the emulator against the port (item 1932; hand-written).

/// `Group::report_speed@00713bb0`, run by `tools/emu/sweep_group_report_speed.py` under unicorn: the speed bookkeeping of a group, every other byte untouched.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_group_report_speed.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (a, want) = super::row(line);
        let (speed, o3c, o40) = (a[0] as i32, a[1] as i32, a[2] as i32);
        let (n3c, n40) = if speed < o40 {
            (speed, speed)
        } else {
            (o3c, o40)
        };
        let got = (i64::from(n40 as u32) << 32) | i64::from(n3c as u32);
        assert_eq!(got, want, "report_speed — row `{line}`");
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("Group::report_speed@00713bb0: {n} rows agree");
}
