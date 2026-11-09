//! `TypeData::time@00663f20` under the emulator against the port (item 2090; hand-written on tools/emu/hooks.py).

use crate::production::time_by;
use crate::type_index::is_spell_type;

/// `TypeData::time@00663f20`, run by `tools/emu/sweep_type_data_time.py` under unicorn (tools/emu): the
/// record's slot `+0x48` the image's own `is_spell_type` (then [`is_spell_type`] of the index) or a stub asked
/// on the record; a spell asks `has_spell(type)` on `leaders + 0x6eec · who`. The port's [`time_by`] is handed
/// the spell test's answer and a `held` that records its ask, so the arms, the ask and its `ecx` are held to
/// the original's calls, and the answer to `eax`.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_type_data_time.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (a, want, tail) = super::tail_row(line);
        let w = |i: usize| i32::try_from(a[i]).expect("a 32-bit word");
        let (base, t, slot, held, who) = (a[0] != 0, w(1), a[2], a[3], w(4));
        let leaders = u32::try_from(a[5]).expect("an address");
        let mut calls = Vec::new();
        let spell = if base {
            is_spell_type(t)
        } else {
            calls.push("is_spell_type,rec".to_string());
            slot != 0
        };
        let got = time_by(
            spell,
            || {
                let ecx = leaders.wrapping_add(who.wrapping_mul(0x6eec) as u32);
                calls.push(format!("has_spell,{ecx},{t}"));
                held != 0
            },
            w(6),
            w(7),
        );
        assert_eq!(tail[0], calls.join(";"), "time — the calls, row `{line}`");
        assert_eq!(i64::from(got), want, "time — eax, row `{line}`");
        n += 1;
    }
    assert!(n >= 400, "only {n} rows");
    eprintln!("TypeData::time@00663f20: {n} rows agree");
}
