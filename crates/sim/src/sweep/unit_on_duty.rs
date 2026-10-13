//! `Unit::on_duty@005fff70` under the emulator against the port (item 2044; hand-written on tools/emu/hooks.py).

/// `Unit::on_duty@005fff70`, run by `tools/emu/sweep_unit_on_duty.py` under unicorn (tools/emu): a type
/// without the combat-role bit (`+0x2c8 & 0x10000`) is no, unasked; past it the unit's activity is asked, a
/// null one is no, and a real one's kind is held to the set `fight.rs`'s `on_duty` matches — `ATTACK_TO`, 5,
/// `GUARD`, `GROUP_ATTACK_TO`, `GROUP_PATROL` (`orders::index`). Each call's `ecx` is held too: the unit, then
/// the order.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    use crate::orders::index::{ATTACK_TO, GROUP_ATTACK_TO, GROUP_PATROL, GUARD};
    let Some(rows) = super::emu_rows("sweep_unit_on_duty.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (lhs, rhs) = line
            .split_once(" -> ")
            .expect("`<args…> -> <eax> | <calls>`");
        let (eax, calls) = rhs.split_once(" | ").unwrap_or((rhs, ""));
        let a: Vec<i64> = lhs
            .split_whitespace()
            .map(|s| s.parse().expect("an integer"))
            .collect();
        let (flags, has, kind) = (a[0] as i32, a[1] != 0, a[2]);
        let role = flags & 0x10000 != 0;
        let duty = [ATTACK_TO, 5, GUARD, GROUP_ATTACK_TO, GROUP_PATROL]
            .iter()
            .any(|&k| i64::from(k) == kind);
        let want_calls = match (role, has) {
            (false, _) => "",
            (true, false) => "activity:1",
            (true, true) => "activity:1,kind:1",
        };
        assert_eq!(
            calls.trim(),
            want_calls,
            "on_duty — the calls, row `{line}`"
        );
        assert_eq!(
            eax.trim().parse::<i64>().expect("an integer"),
            i64::from(role && has && duty),
            "on_duty — row `{line}`"
        );
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("Unit::on_duty@005fff70: {n} rows agree");
}
