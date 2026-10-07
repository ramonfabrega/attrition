//! `Farms::snip@008d9240` under the emulator against `farms::Farm::snip`.

use crate::farms::Farm;

/// The record stride (`0x30 * 4` bytes) and the status grid's offset in it.
const STRIDE: i64 = 0xC0;

/// The packed base-6 grid (`tools/emu/sweep_farms_snip.py`, cell `c` is
/// `state * 6**c`) as a farm.
fn farm_of(mut packed: i64) -> Farm {
    let mut f = Farm::default();
    for c in 0..16 {
        f.state[c] = u8::try_from(packed % 6).expect("a state");
        packed /= 6;
    }
    f
}

fn pack(f: &Farm) -> i64 {
    (0..16).rev().fold(0, |a, c| a * 6 + i64::from(f.state[c]))
}

/// `Farms::snip@008d9240`, run by `tools/emu/sweep_farms_snip.py` under
/// unicorn on four farm records with seeded 4x4 grids of states 0..5. The
/// port's `Farm::snip(cell)` takes the grid index with no bounds and no
/// farm, so the *call site* is modelled here: the original tests `x < 4 &&
/// y < 4` (signed — negatives pass), then touches the byte at
/// `farm*0xC0 + 4*y + x` from farm 0's grid, and this test lands that byte
/// on whichever of the four grids holds it (a negative `x` aliases into the
/// previous row, a `y` of `-48 + k` into the previous farm's row `k`) and
/// calls `snip` on that cell; a byte on no grid (the table's other bytes
/// are stood in as `0x7f`, never 2) changes nothing, which the `which = 9`
/// row asserts of the original. Every row is a farm's grid after the call.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_farms_snip.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (args, want) = super::row(line);
        let [farm, x, y, g0, g1, g2, g3, which] = args[..] else {
            panic!("row `{line}` is not eight arguments");
        };
        let mut farms = [g0, g1, g2, g3].map(farm_of);
        if x < 4 && y < 4 {
            let off = farm * STRIDE + 4 * y + x;
            for (f, fm) in farms.iter_mut().enumerate() {
                let rel = off - f as i64 * STRIDE;
                if (0..16).contains(&rel) {
                    fm.snip(rel as usize);
                }
            }
        }
        let ours = if which == 9 {
            // nothing outside the grids: the port has no such bytes
            0
        } else {
            pack(&farms[which as usize])
        };
        assert_eq!(ours, want, "snip(farm {farm}, x {x}, y {y}) — row `{line}`");
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("Farms::snip@008d9240: {n} rows agree");
}
