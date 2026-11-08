//! `Region::is_coast@00680f90` under the emulator against the port (item 2038; hand-written on tools/emu/hooks.py).

/// `Region::is_coast@00680f90`, run by `tools/emu/sweep_region_is_coast.py` under unicorn (tools/emu): a region
/// coasts itself; a land region (below `0x40`) and a sea region coast when the sea's bit, `r - 0x40`, is set in
/// the land region's sixty-four-bit set at `+0x64` — read from whichever side is the land; two lands or two seas
/// never. `world.rs`'s `is_coast` is the port, its land region's list of seas the bit set.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_region_is_coast.py") else {
        return;
    };
    let set = |lo: i64, hi: i64| u64::from(lo as i32 as u32) | u64::from(hi as i32 as u32) << 32;
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (a, want) = super::row(line);
        let (region, other) = (a[0], a[1]);
        let (mine, theirs) = (set(a[2], a[3]), set(a[4], a[5]));
        let got = if region == other {
            true
        } else if region < 0x40 && other >= 0x40 {
            mine >> (other - 0x40) & 1 != 0
        } else if region >= 0x40 && other < 0x40 {
            theirs >> (region - 0x40) & 1 != 0
        } else {
            false
        };
        assert_eq!(i64::from(got), want, "is_coast — row `{line}`");
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("Region::is_coast@00680f90: {n} rows agree");
}
