//! `GroupData::one_member@0046f820` under the emulator against the port (item 1840).

/// `GroupData::one_member@0046f820`, run by `tools/emu/sweep_group_data_one_member.py` under unicorn
/// (tools/emu): `__thiscall`, `ecx` = the group: 1 when exactly one of the first `num` list entries is active (flag bit 0 of the object at `+8` set and its slot `+0xe8` non-zero), through the real `get_num_cap_const`; 0 when `num <= 0`. Rows `<who> <num> <f0> <v0> … <f5> <v5>`; the list is `[3, 0, 5, 1, 4, 2]`.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_group_data_one_member.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (a, want) = super::row(line);
        let got: i64 = one_member(&a);
        assert_eq!(got, want, "GroupData::one_member — row `{line}`");
        n += 1;
    }
    assert!(n >= 400, "only {n} rows");
    eprintln!("GroupData::one_member@0046f820: {n} rows agree");
}

/// The port: `get_num_cap_const` counts the first `num` list entries (read through the permutation `[3, 0, 5, 1, 4, 2]`) whose object is active (flag bit 0 and a non-zero slot `+0xe8`); one member is exactly one.
fn one_member(a: &[i64]) -> i64 {
    let num = a[1];
    if num <= 0 {
        return 0;
    }
    let mut count = num;
    for &k in &[3usize, 0, 5, 1, 4, 2][..num as usize] {
        if !(a[2 + 2 * k] & 1 != 0 && a[3 + 2 * k] != 0) {
            count -= 1;
        }
    }
    i64::from(count == 1)
}
