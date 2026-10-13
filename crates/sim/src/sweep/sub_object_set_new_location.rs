//! `SubObject::set_new_location@00662680` under the emulator against the port (item 2044; hand-written on tools/emu/hooks.py).

/// `SubObject::set_new_location@00662680`, run by `tools/emu/sweep_sub_object_set_new_location.py` under
/// unicorn (tools/emu): the position is stored masked (`^ 0x63637`) at `+0x10`/`+0x14`, the terrain is asked
/// for the height at the table's words for `x >> 6` and `y >> 6` with a last argument of 1, and its height,
/// masked, is stored at `+0xc`; the answer is 1. `collide.rs`'s `set_new_location` is the port's mover, which
/// keeps positions unmasked and asks no terrain height.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_sub_object_set_new_location.py") else {
        return;
    };
    const MASK: i64 = 0x63637;
    let tab = |c: i64| ((c >> 6) as u64).wrapping_mul(2_654_435_761) >> 5 & 0xffff;
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (lhs, rhs) = line
            .split_once(" -> ")
            .expect("`<args…> -> <eax> | <call> | <fields>`");
        let a: Vec<i64> = lhs
            .split_whitespace()
            .map(|s| s.parse().expect("an integer"))
            .collect();
        let (x, y, z, terrain) = (a[0], a[1], a[2], a[3]);
        let mut parts = rhs.split(" | ");
        let eax = parts.next().expect("eax").trim();
        let call = parts.next().expect("the call").trim();
        let fields = parts.next().expect("the fields").trim();
        assert_eq!(eax, "1", "set_new_location — answer, row `{line}`");
        assert_eq!(
            call,
            format!("{terrain},{},{},1", tab(x), tab(y)),
            "set_new_location — the call, row `{line}`"
        );
        let m = |v: i64| i64::from((v ^ MASK) as i32);
        assert_eq!(
            fields,
            format!("{} {} {}", m(z), m(x), m(y)),
            "set_new_location — the fields, row `{line}`"
        );
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("SubObject::set_new_location@00662680: {n} rows agree");
}
