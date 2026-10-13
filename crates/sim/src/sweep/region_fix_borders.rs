//! `Region::fix_borders@00680f60` under the emulator against the port (item 2020; hand-written on tools/emu/hooks.py).

fn unhex(s: &str) -> Vec<u8> {
    (0..s.len() / 2)
        .map(|i| u8::from_str_radix(&s[2 * i..2 * i + 2], 16).expect("hex"))
        .collect()
}

/// `Region::fix_borders@00680f60`, run by `tools/emu/sweep_region_fix_borders.py` under unicorn (tools/emu):
/// whichever region it is called on, the word at `+0x2c` of every one of the 64 regions (`0x88` bytes each) is
/// zeroed — `city.rs`'s `fix_borders` clears every region's border count the same way — and no other byte of the
/// list, the singleton or `this` moves.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_region_fix_borders.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (lhs, rhs) = line
            .split_once(" -> ")
            .expect("`<same> | <before> -> <eax> | <after>`");
        let (same, before) = lhs.split_once(" | ").expect("`<same> | <before>`");
        let (_, after) = rhs.split_once(" | ").expect("`<eax> | <after>`");
        assert_eq!(
            same.trim(),
            "1",
            "the singleton and this untouched — row {n}"
        );
        let mut m = unhex(before.trim());
        for r in 0..64 {
            let at = r * 0x88 + 0x2c;
            m[at..at + 4].fill(0);
        }
        assert_eq!(m, unhex(after.trim()), "fix_borders — bytes, row {n}");
        n += 1;
    }
    assert!(n >= 40, "only {n} rows");
    eprintln!("Region::fix_borders@00680f60: {n} rows agree");
}
