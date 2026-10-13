//! `UnitData::recharge@0060fdf0` under the emulator against the port (item 2061; hand-written on
//! tools/emu/hooks.py).

/// `UnitData::recharge@0060fdf0`, run by `tools/emu/sweep_unit_data_recharge.py` under unicorn (tools/emu),
/// held to `combat.rs`'s `recharge(base, out_of_supply_ratio, bombard)`: the base is the type's `+0x1f4`; the
/// ratio applies when the type's slot `+0x10c` says so and the unit is not supplied — where "supplied" is the
/// supply answer, asked only when the constants' `+0xc20` and `this+0x6c & 1` are not both set (both set skips
/// the question and counts as unsupplied); the bombard test is `this`'s slot `+0xb8` with `(0x10b, 0)`, or the
/// type's slot `+0x60` with the same when `+0xb8` is the base class's own. The port's answer is the byte the
/// unit stores (`recharging` is a `uchar`), so the low byte is held; the bases drawn mostly fit it. A non-null
/// out-pointer reads -1 after (the supply answer is hooked and writes nothing).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_unit_data_recharge.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (a, eax, tail) = super::tail_row(line);
        let (base, rule, slow, masks2, supplied) =
            (a[0] as i32, a[1] != 0, a[2] != 0, a[3], a[4] != 0);
        let (devirt, bombard, out) = (a[5] != 0, a[6] != 0, a[7] != 0);
        let asks_supply = rule && !(slow && masks2 & 1 != 0);
        let ratio = rule && !(asks_supply && supplied);
        let mut calls = vec!["rule:1"];
        if asks_supply {
            calls.push("supply:1");
        }
        if ratio {
            calls.push(if devirt { "type_is:1" } else { "this_is:1" });
        }
        let word = if out { "-1" } else { "0" };
        assert_eq!(
            tail,
            [calls.join(",").as_str(), word],
            "recharge — the calls, row `{line}`"
        );
        assert_eq!(
            eax & 0xff,
            i64::from(crate::combat::recharge(base, ratio, bombard)),
            "recharge — row `{line}`"
        );
        n += 1;
    }
    assert!(n >= 300, "only {n} rows");
    eprintln!("UnitData::recharge@0060fdf0: {n} rows agree");
}
