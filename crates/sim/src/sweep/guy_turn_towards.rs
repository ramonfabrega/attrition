//! `Guy::turn_towards@005d9720` under the emulator against the port (item 2061; hand-written on
//! tools/emu/hooks.py).

/// `Guy::turn_towards@005d9720`, run by `tools/emu/sweep_guy_turn_towards.py` under unicorn (tools/emu), held
/// to `movement.rs`'s `turn_towards(from, to, rate)`: the answer is the turn still owed; the turn rate is asked
/// (with 1, on the guy) only when the turn is not already within the tolerance — exactly when the port, handed
/// a zero rate, still owes a turn — and the turn itself is handed `(to, the new heading, 1, p3)` either way.
/// The heading is `this+0x18`; the second argument is unread.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    use crate::movement::{Angle, turn_towards};
    let Some(rows) = super::emu_rows("sweep_guy_turn_towards.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (a, eax, tail) = super::tail_row(line);
        let (from, to, rate, p3) = (
            Angle(a[0] as u32 as i32),
            Angle(a[1] as u32 as i32),
            a[2] as u32,
            a[3],
        );
        let (new, owed) = turn_towards(from, to, rate);
        let asks = turn_towards(from, to, 0).1 != 0;
        let turn = format!("turn:1:{}/{}/1/{p3}", to.0 as u32, new.0 as u32);
        let calls = if asks {
            format!("rate:1:1,{turn}")
        } else {
            turn
        };
        assert_eq!(
            tail,
            [calls.as_str()],
            "turn_towards — the calls, row `{line}`"
        );
        assert_eq!(eax as u32, owed, "turn_towards — row `{line}`");
        n += 1;
    }
    assert!(n >= 300, "only {n} rows");
    eprintln!("Guy::turn_towards@005d9720: {n} rows agree");
}
