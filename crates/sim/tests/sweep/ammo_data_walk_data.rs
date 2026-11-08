//! `AmmoData::walk_data@0067ab50` under the emulator against the port (item 3058; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `AmmoData::walk_data@0067ab50`, run by `tools/emu/sweep_obj.py` on its row `ammo_data_walk_data` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 112 random bytes of object, 1 argument(s), `p` windows in order: p0 argument 0; the global cell 0x00c06378 points at 32 random bytes; argument 0 names 16 random bytes; vtable slot +0x0 (2 args) stubbed; vtable slot +0x4 (1 args) stubbed; callee 0x00478360 (0 args) stubbed; callee 0x00478310 (0 args) stubbed; callee 0x009132b0 (1 args) stubbed; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 3 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 4 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 0 writes a value in [0, 1, 2, 3, 4, 256, 257, 5, 16, 48] through its argument 0, printed after the answers (`o` in a place); the stubbed vtable laid at `p0+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 400 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("ammo_data_walk_data") else {
        return;
    };
    let obj = super::Obj {
        name: "walk_data",
        full: "AmmoData::walk_data@0067ab50",
        cells: 1,
        calls: true,
        eax: false,
        min_rows: 400,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, log| {
        let _ = (&a, &m, &c, &p);
        {
            // `a`: 0 the DataWalk pointer, 1..=5 the stubs' answers (a[3] what the recycler pops), 6 the value
            // slot +0 writes through its first argument on every call: `flags` is the dword at this+4, `rolling`
            // the dword at this+5, and the walked byte is a stack local (only its low byte is read).
            let v = a[6];
            let input = rd(&p[0], 4);
            let ammo = |m: &[u8]| rd(m, 0x68);
            let mut l0 = vec!["p0+0x0,m+0x4,m+0x5".to_string()];
            let mut l3 = "-".to_string();
            let mut l4 = "-".to_string();
            let l1 = format!("p0+0x0,{}", rd(&c[0], 0x10).wrapping_add(0x9d8));
            let mut l2 = "-".to_string();
            wr(m, 4, v);
            if m[4] & 3 != 0 {
                l0.push("p0+0x0,m+0x5,m+0x68".to_string());
                wr(m, 5, v);
            }
            l0.push("p0+0x0,sp-0x1,sp+0x0".to_string());
            let local = (v & 0xff) as u8;
            let _ = ammo(m);
            if input == 0 {
                if local != 0 {
                    l4 = format!("{},p0+0x0", ammo(m));
                }
            } else if local == 0 {
                if ammo(m) != 0 {
                    l3 = format!("{}", ammo(m).wrapping_add(0x3c));
                    wr(m, 0x68, 0);
                }
            } else if ammo(m) == 0 {
                l2 = "p0+0x0".to_string();
                wr(m, 0x68, a[3]);
                l4 = format!("{},p0+0x0", a[3]);
            } else {
                l4 = format!("{},p0+0x0", ammo(m));
            }
            log.extend([l0.join("/"), l1, l2, l3, l4]);
            0
        }
    });
}
