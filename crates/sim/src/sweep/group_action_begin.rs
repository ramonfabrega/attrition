//! `Group::action_begin@00714100` under the emulator against the port (item 1724).

/// `Group::action_begin@00714100`, run by `tools/emu/sweep_group_action_begin.py` under unicorn
/// (tools/emu): zeroes the dword at `+0x28`, its neighbours untouched.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_group_action_begin.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (a, want) = super::row(line);
        let expect = if a[0] == 0x28 { 0 } else { -1_515_870_811 };
        assert_eq!(want, expect, "Group::action_begin — row `{line}`");
        n += 1;
    }
    assert!(n >= 3, "only {n} rows");
    eprintln!("Group::action_begin@00714100: {n} rows agree");
}
