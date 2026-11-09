//! `BuildData::calc_gather@0062d360` under the emulator against the port (item 2368; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `BuildData::calc_gather@0062d360`, run by `tools/emu/sweep_obj.py` on its row `build_data_calc_gather` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 160 random bytes of object, 6 argument(s), argument 0 names 16 random bytes; argument 1 names 8 random bytes; argument 2 names 16 random bytes; callee 0x00643440 (2 args) stubbed; vtable slot +0x4c (0 args) stubbed; vtable slot +0xb8 (2 args) stubbed; vtable slot +0x60 (2 args) stubbed; callee 0x0064dda0 (2 args) stubbed; callee 0x00630450 (2 args) stubbed; callee 0x00639e40 (13 args) stubbed; the word at `m+0x18` points at 32 random bytes less `0` (placed); slot +0x4c laid as 0x00472350 or its stub, a flag after the answers; slot +0xb8 laid as 0x00653790 or its stub, a flag after the answers; stub 1 answered from [0, 1, 4] (90%); stub 2 answered from [0, 1] (90%); stub 3 answered from [0, 1] (90%); stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 3 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 4 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 5 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 6 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the recorded calls one token a call, in call order, each headed `sIDX:` (the callee); stub 0 writes a value in -300..300 through its argument 0, printed after the answers (`o` in a place); stub 0 writes a value in -300..300 through its argument 1, printed after the answers (`o` in a place); stub 6 writes a value in -5..9 through its argument 1, printed after the answers (`o` in a place); the stubbed vtable laid at `p3+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("build_data_calc_gather") else {
        return;
    };
    let obj = super::Obj {
        name: "calc_gather",
        full: "BuildData::calc_gather@0062d360",
        cells: 0,
        calls: true,
        eax: false,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, log| {
        let _ = (&a, &m, &c, &p);
        {
            // The port's own `holdings::build_calc_gather_steps`, its callees answered as the
            // stubs were: `tile_corner` (stub 0, `o` the corner it writes), slot +0x4c (stub 1;
            // the base `WallData::is_active` when the first flag is set, `flags & 4` inlined),
            // slot +0xb8 `is(0x1a4, 0)` (stub 2; the base `ObjectData::is` → the type's slot
            // +0x60, stub 3), `count_inside(19, 0x34)` (stub 4), `num_gatherers(1, 1)` (stub
            // 5), and the type's `calc_gather` (stub 6, writing the count `o` through arg 1).
            let rec = std::cell::RefCell::new(Vec::<String>::new());
            let count = crate::holdings::build_calc_gather_steps(
                a[4] != 0,
                || {
                    rec.borrow_mut().push("s0:m+0x0,sp-0x28,sp-0x24".into());
                    (a[13], a[14])
                },
                || {
                    if a[16] != 0 {
                        i32::from(m[8] & 4)
                    } else {
                        rec.borrow_mut().push("s1:m+0x0".into());
                        a[7]
                    }
                },
                || {
                    if a[17] != 0 {
                        rec.borrow_mut().push("s3:p3+0x0,420,0".into());
                        a[9] != 0
                    } else {
                        rec.borrow_mut().push("s2:m+0x0,420,0".into());
                        a[8] != 0
                    }
                },
                || {
                    rec.borrow_mut().push("s4:m+0x0,19,52".into());
                    a[10]
                },
                || {
                    rec.borrow_mut().push("s5:m+0x0,1,1".into());
                    a[11]
                },
                |h| {
                    rec.borrow_mut().push(format!(
                        "s6:p3+0x0,p0+0x0,p1+0x0,p2+0x0,{},{},{},{},{},{},{},{},{},m+0x98",
                        a[3],
                        i16::from_le_bytes([m[0x72], m[0x73]]),
                        m[9],
                        a[5],
                        h.corner.0,
                        h.corner.1,
                        h.active,
                        h.gatherers,
                        h.scholars
                    ));
                    a[15]
                },
            );
            log.extend(rec.into_inner());
            wr(&mut p[1], 0, count);
            0
        }
    });
}
