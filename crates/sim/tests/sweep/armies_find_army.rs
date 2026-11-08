//! `Armies::find_army@006f34f0` under the emulator against the port (item 3150; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `Armies::find_army@006f34f0`, run by `tools/emu/sweep_obj.py` on its row `armies_find_army` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 16 random bytes of object, 6 argument(s), `p` windows in order: p0..p26 the place(s), in flag order; the global cell 0x00c06188 points at 16 random bytes; 4 random bytes at 0x00cb4bac; callee 0x006b52e0 (2 args) stubbed; callee 0x0046cff0 (0 args) stubbed; callee 0x006f9120 (2 args) stubbed; vtable slot +0xcc (0 args) stubbed; vtable slot +0xc4 (0 args) stubbed; the word at `0xc09710` points at 64 random bytes less `0` (placed); the word at `p0+0` points at 64 random bytes less `0` (placed); the word at `p0+4` points at 64 random bytes less `0` (placed); the word at `p0+8` points at 64 random bytes less `0` (placed); the word at `p0+12` points at 64 random bytes less `0` (placed); the word at `p0+16` points at 64 random bytes less `0` (placed); the word at `p0+20` points at 64 random bytes less `0` (placed); the word at `p0+24` points at 64 random bytes less `0` (placed); the word at `p0+28` points at 64 random bytes less `0` (placed); the word at `p0+32` points at 64 random bytes less `0` (placed); the word at `p0+36` points at 64 random bytes less `0` (placed); the word at `p0+40` points at 64 random bytes less `0` (placed); the word at `p0+44` points at 64 random bytes less `0` (placed); the word at `p0+48` points at 64 random bytes less `0` (placed); the word at `p0+52` points at 64 random bytes less `0` (placed); the word at `p0+56` points at 64 random bytes less `0` (placed); the word at `p0+60` points at 64 random bytes less `0` (placed); the word at `0xc0aec0` points at 16 random bytes less `0` (placed); the word at `p17+0` points at 32 random bytes less `0` (placed); the word at `p17+4` points at 32 random bytes less `0` (placed); the word at `p17+8` points at 32 random bytes less `0` (placed); the word at `p17+12` points at 32 random bytes less `0` (placed); the word at `0xcae5fc` points at 1024 random bytes less `0` (placed); the word at `p18+0x18` points at 704 random bytes less `0` (placed); the word at `p19+0x18` points at 704 random bytes less `0` (placed); the word at `p20+0x18` points at 704 random bytes less `0` (placed); the word at `p21+0x18` points at 704 random bytes less `0` (placed); slot +0xcc laid as 0x0046ce80 or its stub, a flag after the answers; slot +0xc4 laid as 0x0046ce60 or its stub, a flag after the answers; stub 0 answered from [0, 1, 2] (90%); stub 3 answered from [0, 1] (90%); stub 4 answered from [0, 1] (90%); stub 1 answers 0..150 every row; stub 2 answers 0..4 every row; stub 1 records its `ecx`, `edx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx`, `edx` and arguments, a window's pointer by its name, after ` # `; the stubbed vtable laid at `p18+0` as well as at `this`; the stubbed vtable laid at `p19+0` as well as at `this`; the stubbed vtable laid at `p20+0` as well as at `this`; the stubbed vtable laid at `p21+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 600 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("armies_find_army") else {
        return;
    };
    let obj = super::Obj {
        name: "find_army",
        full: "Armies::find_army@006f34f0",
        cells: 2,
        calls: true,
        eax: true,
        min_rows: 600,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, log| {
        let _ = (&a, &m, &c, &p);
        let rd =
            |w: &[u8], o: usize| i32::from_le_bytes(w[o..o + 4].try_into().expect("four bytes"));
        // c0 the world cell, c1 `find_dist`; p1..p16 the armies, p18..p21 the units and p23..p26 their data;
        // a[6] the region `get_tregion` answers, a[7] `vector_dist`, a[8] `Army::count`, a[9] and a[10] the
        // slot +0xcc / +0xc4 answers, a[11] and a[12] the flags laying `is_supply` / `is_hero`.
        let tre = a[6];
        let mut best = -1i32;
        let mut near = 99_999_999i32;
        c[1][..4].copy_from_slice(&near.to_le_bytes());
        let (mut l1, mut l2) = (Vec::new(), Vec::new());
        for k in 0..16usize {
            let r = &p[1 + k];
            if i16::from_le_bytes([r[0], r[1]]) == 0 || rd(r, 8) != tre {
                continue;
            }
            let dx = a[1].wrapping_sub(rd(r, 0x38)).wrapping_abs();
            let dy = a[2].wrapping_sub(rd(r, 0x3c)).wrapping_abs();
            l1.push(format!("{dx},{dy}"));
            let dist = a[7];
            if near < dist || (a[3] >= 0 && a[3] < dist) {
                continue;
            }
            let mut take = a[5] < 0;
            if !take {
                let u = a[5] as usize;
                let flags = rd(&p[23 + u], 0x2b8);
                let supply = if a[11] != 0 {
                    flags & 0x40 != 0
                } else {
                    a[9] != 0
                };
                if supply {
                    l2.push(format!("p{}+0x0,{},17,63", 1 + k, a[5]));
                    take = a[8] <= 2;
                } else {
                    let hero = if a[12] != 0 {
                        flags & 0x20 != 0
                    } else {
                        a[10] != 0
                    };
                    if hero {
                        l2.push(format!("p{}+0x0,{},17,54", 1 + k, a[5]));
                        take = a[8] <= 1;
                    } else {
                        take = true;
                    }
                }
            }
            if take {
                best = k as i32;
                near = dist;
                c[1][..4].copy_from_slice(&near.to_le_bytes());
            }
        }
        let j = |l: Vec<String>| {
            if l.is_empty() {
                "-".to_string()
            } else {
                l.join("/")
            }
        };
        log.push(j(l1));
        log.push(j(l2));
        i64::from(best)
    });
}
