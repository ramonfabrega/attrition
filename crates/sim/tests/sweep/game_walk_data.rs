//! `Game::walk_data@00589600` under the emulator against the port (item 3058; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `Game::walk_data@00589600`, run by `tools/emu/sweep_obj.py` on its row `game_walk_data` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 32 random bytes of object, 1 argument(s), `p` windows in order: p0 argument 0; the global cell 0x00c06378 points at 32 random bytes; the global cell 0x00c061ec points at 2304 random bytes; argument 0 names 16 random bytes; vtable slot +0x4 (1 args) stubbed; callee 0x005d6570 (1 args) stubbed; vtable slot +0x0 (2 args) stubbed; the import cell 0x00ac5460 holds a stub (0 args); stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 3 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 3 is `__cdecl`: the caller pops its arguments; the stubbed vtable laid at `p0+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 400 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("game_walk_data") else {
        return;
    };
    let obj = super::Obj {
        name: "walk_data",
        full: "Game::walk_data@00589600",
        cells: 2,
        calls: true,
        eax: false,
        min_rows: 400,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, log| {
        let _ = (&a, &m, &c, &p);
        {
            // `a`: 0 the DataWalk pointer, 1..=4 the stubs' answers (a[4] is `timeGetTime`). The walker's callbacks
            // write nothing here; `c1` is the game record, `c0` the string table cell.
            let (input, checksum) = (rd(&p[0], 4), rd(&p[0], 8));
            let l0 = format!("p0+0x0,{}", rd(&c[0], 0x10).wrapping_add(0xd980));
            let l1 = "c1+0xc,p0+0x0".to_string();
            let mut calls = vec!["p0+0x0,c1+0x550,c1+0x6e4".to_string()];
            let mut l3 = "-".to_string();
            if checksum == 0 {
                calls.push("p0+0x0,c1+0x814,c1+0x81c".to_string());
                calls.push(format!(
                    "p0+0x0,c1+0x820,c1+{:#x}",
                    0x820 + rd(&c[1], 0x818)
                ));
                if input != 0 {
                    wr(&mut c[1], 0x81c, 2);
                }
            }
            if input != 0 {
                c[1][0x821] &= 0xfe;
                if rd(&c[1], 0x81c) == 0 {
                    wr(&mut c[1], 0x81c, 2);
                }
            }
            if checksum == 0 {
                calls.push("p0+0x0,c1+0x844,c1+0x848".to_string());
                l3 = "p0+0x0".to_string();
                wr(&mut c[1], 0x840, a[4]);
            }
            log.extend([l0, l1, calls.join("/"), l3]);
            0
        }
    });
}
