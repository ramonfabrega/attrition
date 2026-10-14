//! `UnitData::in_supply@00609ef0` under the emulator against the port (item 2217; tools/sweep_gen.py, obj with windows).

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

/// `UnitData::in_supply@00609ef0`, run by `tools/emu/sweep_unit_data_in_supply.py` under unicorn
/// (tools/emu): 32 random bytes of object, 2 argument(s), the global cell 0x00cae5fc points at 1024 random bytes; the global cell 0x00c061d0 points at 320 random bytes; argument 1 names 4 random bytes; callee 0x0073aba0 (3 args) stubbed; the word at `m+0x18` points at 544 random bytes less `0` (placed); the word at `c1+0x134` points at 28 random bytes less `(i32(c[1],0)*i32(c[0],4*((i32(m,0x14)^0x63637)>>8))+i32(c[0],4*((i32(m,0x10)^0x63637)>>8)))*0x1c` (placed); stub 0 answered from [-1, 0, 1, 5, -5] (90%); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_unit_data_in_supply.py") else {
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
        let mut p: Vec<Vec<u8>> = wins.split_off(2);
        let mut c: Vec<Vec<u8>> = wins;
        let want_eax: i64 = eax.trim().parse().expect("an integer");
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            let _ = (&a, &m, &c, &p);
            {
    use crate::world::Owner;
    let land = rd(&p[1], 0x218) == 0;
    let domain = if land { crate::attrition::Domain::Land } else { crate::attrition::Domain::Sea };
    let o = p[2][0xf] as i8;
    let ground = if o >= 0 { Owner::Player(o as u8) } else { Owner::None };
    let who = m[9];
    let found = a[2] >= 0;
    // The out-pointer: -1, then the supply `find_supply` found when it was asked.
    wr(&mut p[0], 0, -1);
    if land && !matches!(ground, Owner::Player(q) if q == who) && found {
        wr(&mut p[0], 0, a[2]);
    }
    i64::from(crate::supply::in_supply(domain, ground, who, found))
}
        };
        assert_eq!(got, want_eax, "in_supply — eax, row `{line}`");
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "in_supply — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "in_supply — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[3 + i], "in_supply — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("UnitData::in_supply@00609ef0: {n} rows agree");
}
