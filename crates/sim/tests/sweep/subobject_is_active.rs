//! `SubObjectData::is_active@0046cda0` under the emulator against the port (item 1705).

use crate::flag_reads::sub_object_is_active;

/// `SubObjectData::is_active@0046cda0`, run by `tools/emu/sweep_subobject_is_active.py` under unicorn
/// (tools/emu): `flags` (byte at `+0x8`) `& 1`, over every bit edge and a seeded band, against `flag_reads::sub_object_is_active` — what is proved is the bit's identity (0x1), that the other bits and the sign do not leak, and that the original returns the masked value itself.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_subobject_is_active.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (a, want) = super::row(line);
        let word = u8::try_from(a[0]).expect("a u8");
        assert_eq!(
            sub_object_is_active(word),
            want != 0,
            "sub_object_is_active — row `{line}`"
        );
        assert_eq!(
            want,
            i64::from(word & 0x1),
            "sub_object_is_active — returns the masked word, row `{line}`"
        );
        n += 1;
    }
    assert!(n >= 256, "only {n} rows");
    eprintln!("SubObjectData::is_active@0046cda0: {n} rows agree");
}
