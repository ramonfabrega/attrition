//! `TypeData::research_time@006639a0` under the emulator against the port (item 2090; hand-written on tools/emu/hooks.py).

use crate::production::research_time_by;
use crate::type_index::is_spell_type;

/// `TypeData::research_time@006639a0`, run by `tools/emu/sweep_type_data_research_time.py` under unicorn
/// (tools/emu): the spell test as `time`'s; a spell costs `res_time · 100`, anything else asks the record's slot
/// `+0x6c` with `who` and scales it by the premium at `[0xc061e4] + 0x3a8`. The port's [`research_time_by`] is
/// handed a `time` that records its ask, so the arms, the ask and the logical shift are held to the original.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_type_data_research_time.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (a, want, tail) = super::tail_row(line);
        let w = |i: usize| i32::try_from(a[i]).expect("a 32-bit word");
        let (base, t, slot, who) = (a[0] != 0, w(1), a[2], w(4));
        let mut calls = Vec::new();
        let spell = if base {
            is_spell_type(t)
        } else {
            calls.push("is_spell_type,rec".to_string());
            slot != 0
        };
        let got = research_time_by(
            spell,
            || {
                calls.push(format!("time,rec,{who}"));
                w(3)
            },
            w(5),
            w(6),
        );
        assert_eq!(
            tail[0],
            calls.join(";"),
            "research_time — the calls, row `{line}`"
        );
        assert_eq!(i64::from(got), want, "research_time — eax, row `{line}`");
        n += 1;
    }
    assert!(n >= 400, "only {n} rows");
    eprintln!("TypeData::research_time@006639a0: {n} rows agree");
}
