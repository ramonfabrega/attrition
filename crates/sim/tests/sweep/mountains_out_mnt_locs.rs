//! `MountainsOut::mnt_locs@0089be90` under the emulator against the port (item 1856).

/// `MountainsOut::mnt_locs@0089be90`, run by `tools/emu/sweep_mountains_out_mnt_locs.py` under unicorn
/// (tools/emu): `__thiscall`, no arguments: `[[this]+8] + 0xa0 + this` — the vtable's slot-8 word is a displacement. Row `<this> <disp> -> <result>`.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_mountains_out_mnt_locs.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (a, want) = super::row(line);
        let _ = &a;
        let got: i64 = (a[0] as i32).wrapping_add(a[1] as i32).wrapping_add(0xa0) as i64;
        assert_eq!(got, want, "MountainsOut::mnt_locs — row `{line}`");
        n += 1;
    }
    assert!(n >= 100, "only {n} rows");
    eprintln!("MountainsOut::mnt_locs@0089be90: {n} rows agree");
}
