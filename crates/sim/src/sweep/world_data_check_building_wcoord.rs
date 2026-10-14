//! `WorldData::check_building_wcoord@006b26e0` under the emulator against the port (item 2217; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

fn unhex(s: &str) -> Vec<u8> {
    (0..s.len() / 2)
        .map(|i| u8::from_str_radix(&s[2 * i..2 * i + 2], 16).expect("hex"))
        .collect()
}

/// `WorldData::check_building_wcoord@006b26e0`, run by `tools/emu/sweep_world_data_check_building_wcoord.py` under unicorn
/// (tools/emu): 16 random bytes of object, 8 argument(s), the global cell 0x00c06188 points at 320 random bytes; argument 0 names 4 random bytes; argument 1 names 4 random bytes; callee 0x006b27f0 (5 args) stubbed; the word at `c0+0x134` points at 28 random bytes less `(i32(c[0],0)*i32(p[1],0)+i32(p[0],0))*0x1c` (placed); stub 0 answered from [0, 0, 1, 1, 2, 2, 3, 3, 4, 5, -1] (90%); stub 0 answers 49 values in call order, printed after the answers; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_world_data_check_building_wcoord.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (lhs, rhs) = line.split_once(" -> ").expect("a row is `<args…> | <windows> -> <eax> | <windows>`");
        let (args, before) = lhs.split_once(" | ").expect("`<args> | <before>`");
        let (eax, after) = rhs.split_once(" | ").expect("`<eax> | <after>`");
        let a: Vec<i32> = args.split_whitespace().map(|s| s.parse().expect("an integer")).collect();
        let mut wins: Vec<Vec<u8>> = before.trim().split(';').map(unhex).collect();
        let mut m = wins.remove(0);
        let mut p: Vec<Vec<u8>> = wins.split_off(1);
        let mut c: Vec<Vec<u8>> = wins;
        let want_eax: i64 = eax.trim().parse().expect("an integer");
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            let _ = (&a, &m, &c, &p);
            {
    let cell = &p[2];
    let o = cell[0xf] as i8;
    let owner = (o >= 0).then_some(o as u8);
    let flags = u16::from_le_bytes([cell[0], cell[1]]);
    if crate::ai_place::wcoord_cell_refused(owner, a[2] as u8, flags, cell[0x11]) {
        0
    } else {
        // `space_at_corner` answers each corner in turn (`--seq`, a[9..58]).
        let mut k = 0usize;
        i64::from(crate::ai_place::best_corner(a[4], a[5], a[6], |_, _| {
            let v = a[9 + k.min(48)];
            k += 1;
            v
        }))
    }
}
        };
        assert_eq!(got, want_eax, "check_building_wcoord — eax, row `{line}`");
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "check_building_wcoord — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "check_building_wcoord — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[2 + i], "check_building_wcoord — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("WorldData::check_building_wcoord@006b26e0: {n} rows agree");
}
