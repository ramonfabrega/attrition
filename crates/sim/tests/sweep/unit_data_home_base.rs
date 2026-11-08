//! `UnitData::home_base@00609dc0` under the emulator against the port (item 3166; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `UnitData::home_base@00609dc0`, run by `tools/emu/sweep_obj.py` on its row `unit_data_home_base` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 1024 random bytes of object, 1 argument(s), `p` windows in order: p0 argument 0, p1..p12 the place(s), in flag order, p13..p14 the stubs' answer window(s); the global cell 0x00c0ab84 points at 16 random bytes; the global cell 0x00c0aba0 points at 16 random bytes; argument 0 names 4 random bytes (null a third of the rows); callee 0x00651a80 (1 args) stubbed; vtable slot +0x10 (0 args) stubbed; vtable slot +0x11c (0 args) stubbed; vtable slot +0x30 (0 args) stubbed; vtable slot +0xfc (0 args) stubbed; vtable slot +0x4c (0 args) stubbed; callee 0x00645e00 (1 args) stubbed; callee 0x0046d700 (0 args) stubbed; callee 0x0046d6e0 (0 args) stubbed; the word at `m+0xdc` points at 16 random bytes less `0` (placed, null a third of the rows); the word at `p1+4` points at 16 random bytes less `0` (placed); the word at `p2+8` points at 16 random bytes less `0` (placed); the word at `m+0x18` points at 16 random bytes less `0` (placed); the word at `c0+0` points at 16 random bytes less `0` (placed); the word at `c0+4` points at 16 random bytes less `0` (placed); the word at `c0+8` points at 16 random bytes less `0` (placed); the word at `c0+12` points at 16 random bytes less `0` (placed); the word at `c1+0` points at 16 random bytes less `0` (placed); the word at `c1+4` points at 16 random bytes less `0` (placed); the word at `c1+8` points at 16 random bytes less `0` (placed); the word at `c1+12` points at 16 random bytes less `0` (placed); stub 0 answered from [-1, -1, -1, -1, 2] (90%); stub 1 answered from [25, 25, 25, 5, 3] (90%); stub 3 answered from [1, 1, 1, 0] (90%); stub 5 answered from [1, 1, 1, 0] (90%); stub 6 answered from [1, 1, 1, 0] (90%); stub 2 answers the address of 48 random bytes, a `p` window after the places; stub 4 answers the address of 16 random bytes, a `p` window after the places; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 6 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 7 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 8 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the stubbed vtable laid at `p3+0` as well as at `this`; the stubbed vtable laid at `p5+0` as well as at `this`; the stubbed vtable laid at `p6+0` as well as at `this`; the stubbed vtable laid at `p7+0` as well as at `this`; the stubbed vtable laid at `p8+0` as well as at `this`; the stubbed vtable laid at `p9+0` as well as at `this`; the stubbed vtable laid at `p10+0` as well as at `this`; the stubbed vtable laid at `p11+0` as well as at `this`; the stubbed vtable laid at `p12+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 600 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("unit_data_home_base") else {
        return;
    };
    let obj = super::Obj {
        name: "home_base",
        full: "UnitData::home_base@00609dc0",
        cells: 2,
        calls: true,
        eax: true,
        min_rows: 600,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, log| {
        let _ = (&a, &m, &c, &p);
        // p0 the out-word, p1 the list head, p2 its node, p3 the order, p4 the `m+0x18` record, p5..p8 the
        // player-0 objects and p9..p12 the player-1 ones, p13 slot +0x11c's answer, p14 slot +0xfc's;
        // a[1] get_inside, a[2] the order's type, a[4] slot +0x30, a[6] slot +0x4c, a[7] can_carry.
        let rdw =
            |w: &[u8], o: usize| i32::from_le_bytes(w[o..o + 4].try_into().expect("four bytes"));
        let mut toks = [
            if a[0] == 0 { "m+0x0,0" } else { "m+0x0,p0+0x0" }.to_string(),
            "-".to_string(),
            "-".to_string(),
            "-".to_string(),
        ];
        let r = 'body: {
            if a[1] >= 0 {
                break 'body a[1] as i64;
            }
            if rdw(m, 0xdc) == 0 {
                break 'body -1;
            }
            wr(m, 0xd4, rdw(&p[1], 4));
            wr(m, 0xcc, rdw(&p[2], 8));
            m[0xd0] = p[2][0xc];
            if a[2] == 0x19 {
                match rdw(&p[13], 8) {
                    0 => break 'body rdw(&p[13], 0x1c) as i64,
                    1 => {
                        toks[2] = "m+0xcc".to_string();
                        toks[3] = "m+0xcc".to_string();
                    }
                    _ => {}
                }
            }
            if a[4] == 0 {
                break 'body -1;
            }
            if a[0] != 0 {
                let v = rdw(&p[14], 8);
                wr(&mut p[0], 0, v);
            }
            let (idx, pl) = (rdw(&p[14], 4), rdw(&p[14], 8));
            if idx < 0 || a[6] == 0 {
                break 'body -1;
            }
            let obj = 5 + 4 * pl + idx;
            toks[1] = format!("p{obj}+0x0,{}", rdw(&p[4], 4));
            if a[7] == 0 {
                break 'body -1;
            }
            idx as i64
        };
        for t in toks {
            log.push(t);
        }
        r
    });
}
