//! `UnitData::is_seen@00607a60` under the emulator against the port (item 2877; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `UnitData::is_seen@00607a60`, run by `tools/emu/sweep_obj.py` on its row `unit_data_is_seen` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 128 random bytes of object, 2 argument(s), the global cell 0x00c061d0 points at 16 random bytes; the global cell 0x00cae5fc points at 1024 random bytes; the global cell 0x00c061e8 points at 64 random bytes; callee 0x0060b030 (0 args) stubbed; callee 0x0060a630 (1 args) stubbed; callee 0x006b55c0 (3 args) stubbed; the word at `m+0x18` points at 704 random bytes less `0` (placed); stub 0 answered from [0, 1] (90%); stub 1 answered from [0, 1] (90%); stub 2 answered from [0, 1] (90%); stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 prints the 4 bytes its argument 0 points at, hex, after the record (`/` a call); stub 2 prints the 4 bytes its argument 1 points at, hex, after the record (`/` a call); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("unit_data_is_seen") else {
        return;
    };
    let obj = super::Obj {
        name: "is_seen",
        full: "UnitData::is_seen@00607a60",
        cells: 3,
        calls: true,
        eax: true,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, log| {
        let _ = (&a, &m, &c, &p);
        {
            let (player, mode) = (a[0], a[1]);
            let masks = rd(m, 0x68) as u32;
            let masks2 = rd(m, 0x6c) as u32;
            let flags = rd(&p[0], 0x2b4) as u32;
            let mut l: [String; 5] = std::array::from_fn(|_| "-".to_string());
            let mut to_check = true;
            if mode == 0 {
                let mut pass = false;
                if masks & 0x800 != 0 || flags & 0x4000 != 0 || masks2 & 0x8000 != 0 {
                    pass = true;
                } else if flags & 0x40000 != 0 {
                    l[0] = "m+0x0".into();
                    pass = a[2] == 0;
                }
                if pass && masks & 0x1000 == 0 {
                    l[1] = format!("m+0x0,{player}");
                    if a[3] == 0 {
                        to_check = false;
                    }
                }
            }
            let eax: i64 = if !to_check {
                0
            } else if player != i32::from(m[9]) && c[2][0x30] != 3 {
                let d = |v: i32| rd(&c[1], (((v ^ 0x63637) >> 7) as usize) * 4);
                let h = |v: i32| {
                    v.to_le_bytes()
                        .iter()
                        .map(|b| format!("{b:02x}"))
                        .collect::<String>()
                };
                l[2] = format!("c0+0x0,sp-0x8,sp-0x4,{player}");
                l[3] = h(d(rd(m, 0x10)));
                l[4] = h(d(rd(m, 0x14)));
                if a[4] != 0 {
                    1
                } else {
                    i64::from(
                        (1i32.wrapping_shl(player as u32 & 31) & i32::from(m[0x40] as i8)) != 0,
                    )
                }
            } else {
                1
            };
            log.extend(l);
            eax
        }
    });
}
