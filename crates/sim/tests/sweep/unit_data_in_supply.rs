//! `UnitData::in_supply@00609ef0` under the emulator against the port (item 2217; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `UnitData::in_supply@00609ef0`, run by `tools/emu/sweep_obj.py` on its row `unit_data_in_supply` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 32 random bytes of object, 2 argument(s), the global cell 0x00cae5fc points at 1024 random bytes; the global cell 0x00c061d0 points at 320 random bytes; argument 1 names 4 random bytes; callee 0x0073aba0 (3 args) stubbed; the word at `m+0x18` points at 544 random bytes less `0` (placed); the word at `c1+0x134` points at 28 random bytes less `(i32(c[1],0)*i32(c[0],4*((i32(m,0x14)^0x63637)>>8))+i32(c[0],4*((i32(m,0x10)^0x63637)>>8)))*0x1c` (placed); stub 0 answered from [-1, 0, 1, 5, -5] (90%); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("unit_data_in_supply") else {
        return;
    };
    let obj = super::Obj {
        name: "in_supply",
        full: "UnitData::in_supply@00609ef0",
        cells: 2,
        calls: false,
        eax: true,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, _log| {
        let _ = (&a, &m, &c, &p);
        {
            use crate::world::Owner;
            let land = rd(&p[1], 0x218) == 0;
            let domain = if land {
                crate::attrition::Domain::Land
            } else {
                crate::attrition::Domain::Sea
            };
            let o = p[2][0xf] as i8;
            let ground = if o >= 0 {
                Owner::Player(o as u8)
            } else {
                Owner::None
            };
            let who = m[9];
            let found = a[2] >= 0;
            // The out-pointer: -1, then the supply `find_supply` found when it was asked.
            wr(&mut p[0], 0, -1);
            if land && !matches!(ground, Owner::Player(q) if q == who) && found {
                wr(&mut p[0], 0, a[2]);
            }
            i64::from(crate::supply::in_supply(domain, ground, who, found))
        }
    });
}
