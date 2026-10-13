//! `MiningList::clear@00471160` under the emulator against the port (item 1724).

/// `MiningList::clear@00471160`, run by `tools/emu/sweep_mining_list_clear.py` under unicorn
/// (tools/emu): the dword at `+4` becomes 0, the bytes `mtn` (`+0x1c`) and `cliff` (`+0x1d`) become 0xff, every other byte of the 0xa5 fill stays; the decompile's `_padding_` is that dword.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_mining_list_clear.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (a, want) = super::row(line);
        let expect = match a[0] {
            4..=7 => 0,
            0x1c | 0x1d => 255,
            _ => 0xa5,
        };
        assert_eq!(want, expect, "MiningList::clear — row `{line}`");
        n += 1;
    }
    assert!(n >= 32, "only {n} rows");
    eprintln!("MiningList::clear@00471160: {n} rows agree");
}
