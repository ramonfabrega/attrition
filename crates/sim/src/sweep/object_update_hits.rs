//! `Object::update_hits@00647010` under the emulator against the port (item 2044; hand-written on tools/emu/hooks.py).

/// `Object::update_hits@00647010`, run by `tools/emu/sweep_object_update_hits.py` under unicorn (tools/emu): a
/// Citizen (`cast.rs`'s `PEASANTS`, or `0x33`) takes the hits of the newest militia line its leader's byte
/// holds — `0x10` the Partisan (`MILITIA + 2`), `8` the Minuteman (`MILITIA + 1`), `4` the Militia — and any
/// other type its own; the answer is stored at `this+0x20` as well. `cast.rs`'s `type_hits` is the port, the
/// leader's bits its `holds_unit_bit`.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    use crate::cast::{MILITIA, PEASANTS};
    let Some(rows) = super::emu_rows("sweep_object_update_hits.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (lhs, rhs) = line
            .split_once(" -> ")
            .expect("`<args…> -> <eax> | <stored>`");
        let (eax, stored) = rhs.split_once(" | ").expect("`<eax> | <stored>`");
        let a: Vec<i64> = lhs
            .split_whitespace()
            .map(|s| s.parse().expect("an integer"))
            .collect();
        let (kind, bits) = (a[1] as i32, a[3]);
        let hits = |k: i32| match k - MILITIA {
            0 => a[5],
            1 => a[6],
            2 => a[7],
            _ => a[4],
        };
        let want = if matches!(kind, PEASANTS | 0x33) {
            if bits & 0x10 != 0 {
                hits(MILITIA + 2)
            } else if bits & 8 != 0 {
                hits(MILITIA + 1)
            } else if bits & 4 != 0 {
                hits(MILITIA)
            } else {
                a[4]
            }
        } else {
            a[4]
        };
        let eax: i64 = eax.trim().parse().expect("an integer");
        let stored: i64 = stored.trim().parse().expect("an integer");
        assert_eq!((eax, stored), (want, want), "update_hits — row `{line}`");
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("Object::update_hits@00647010: {n} rows agree");
}
