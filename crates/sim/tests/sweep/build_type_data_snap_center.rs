//! `BuildTypeData::snap_center@00636190` under the emulator against the port (item 3239; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `BuildTypeData::snap_center@00636190`, run by `tools/emu/sweep_obj.py` on its row `build_type_data_snap_center` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 576 random bytes of object, 5 argument(s), `p` windows in order: p0..p1 pointed argument(s), p2 the place; the global cell 0x00c061d0 points at 1024 random bytes; the global cell 0x00cae5fc points at 2048 random bytes; 324 random bytes at 0x00adc400 (each word in -2..2); 324 random bytes at 0x00adcaf0 (each word in -2..2); argument 2 names 4 random bytes; argument 3 names 4 random bytes; vtable slot +0x60 (2 args) stubbed; callee 0x006b54f0 (3 args) stubbed; callee 0x00636a50 (5 args) stubbed; the word at `c0+0x134` points at 1024 random bytes less `0` (placed); stub 0 answered from [0, 1] (90%); stub 1 answered from [0, 0, 0, 0, 0, 0, 0, 1] (90%); stub 2 answered from [0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1] (90%); stub 0 answers 5 values in call order, printed after the answers; stub 1 answers 8 values in call order, printed after the answers; stub 2 answers 81 values in call order, printed after the answers; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 prints the 4 bytes its argument 0 points at, hex, after the record (`/` a call); stub 1 prints the 4 bytes its argument 1 points at, hex, after the record (`/` a call); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 6000 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("build_type_data_snap_center") else {
        return;
    };
    let obj = super::Obj {
        name: "snap_center",
        full: "BuildTypeData::snap_center@00636190",
        cells: 4,
        calls: true,
        eax: false,
        min_rows: 6000,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, log| {
        let _ = (&a, &m, &c, &p);
        {
            // `BuildTypeData::snap_center@00636190(x, y, &out_x, &out_y, who)` as the original ends
            // it: the crate's `Sim::snap_center_placed` takes a whole `Sim`, so the byte windows
            // cannot call it; this is the listing's port, oil arm included (the crate's SEAM).
            // a[0..5] the arguments, a[5..8] the stubs' one answer (unread), a[8..13] the `is`
            // slot's answers in call order (stub 0), a[13..21] `was_really_seen`'s (stub 1),
            // a[21..102] `blocked_site`'s (stub 2). c0 the world, c1 `div_3_table`, c2 `move_y`
            // (0xadc400), c3 `move_x` (0xadcaf0), p2 the world's cell array (stride 0x1c).
            let (mut x, mut y, who) = (a[0], a[1], a[4]);
            let dv = |k: i32| rd(&c[1], 4 * usize::try_from(k).expect("table index"));
            let (mvy, mvx) = (|i: usize| rd(&c[2], 4 * i), |i: usize| rd(&c[3], 4 * i));
            let (w, h, rows, cols) = (rd(&c[0], 0), rd(&c[0], 4), rd(&c[0], 0x18), rd(&c[0], 0x1c));
            let seen = |tx: i32, ty: i32| {
                let o = usize::try_from(w * ty + tx).expect("cell") * 0x1c;
                u16::from_le_bytes([p[2][o], p[2][o + 1]]) & 0x800 != 0
            };
            let (mut n0, mut n1, mut n2) = (0usize, 0usize, 0usize);
            let (mut l0, mut l1, mut l2): (Vec<String>, Vec<String>, Vec<String>) =
                (Vec::new(), Vec::new(), Vec::new());
            let (mut dx, mut dy): (Vec<String>, Vec<String>) = (Vec::new(), Vec::new());
            let mut is = |t: i32| {
                let r = a[8 + n0.min(4)] != 0;
                n0 += 1;
                l0.push(format!("m+0x0,{t},0"));
                r
            };
            let hex = |v: i32| {
                v.to_le_bytes()
                    .iter()
                    .map(|b| format!("{b:02x}"))
                    .collect::<String>()
            };
            if is(0x1a5) || is(0x1a6) {
                let (tx, ty) = (dv(x >> 8), dv(y >> 8));
                if tx >= 0 && ty >= 0 && tx < w && ty < h && seen(tx, ty) {
                    x = tx * 0x300 + 0x180;
                    y = ty * 0x300 + 0x180;
                } else {
                    for i in 1..=8 {
                        let (nx, ny) = (mvx(i) + tx, mvy(i) + ty);
                        if nx < 0 || ny < 0 || nx >= w || ny >= h || !seen(nx, ny) {
                            continue;
                        }
                        l1.push(format!("c0+0x0,sp-0x30,sp-0x2c,{who}"));
                        dx.push(hex(2 * nx + 1));
                        dy.push(hex(2 * ny + 1));
                        let r = a[13 + n1.min(7)] != 0;
                        n1 += 1;
                        if r {
                            x = nx * 0x300 + 0x180;
                            y = ny * 0x300 + 0x180;
                            break;
                        }
                    }
                }
            }
            let mut blocked = |bx: i32, by: i32| {
                let r = a[21 + n2.min(80)] != 0;
                n2 += 1;
                l2.push(format!("m+0x0,{bx},{by},{who},-1,0"));
                r
            };
            if is(0x1b0) || (who >= 0 && (is(0x1a3) || is(0x1a2))) {
                let (tx, ty) = (dv(x >> 6), dv(y >> 6));
                if blocked(tx * 0xc0 + 0x60, ty * 0xc0 + 0x60) {
                    for i in 1..=80 {
                        let ny = mvy(i) + ty;
                        if ny < 0 || ny >= rows || ny >= cols {
                            continue;
                        }
                        let nx = mvx(i) + tx;
                        if !blocked(nx * 0xc0 + 0x60, ny * 0xc0 + 0x60) {
                            x = nx * 0xc0 + 0x60;
                            y = ny * 0xc0 + 0x60;
                            break;
                        }
                    }
                }
            }
            let (tx, ty) = (dv(x >> 6), dv(y >> 6));
            wr(
                &mut p[0],
                0,
                tx * 0xc0 + if rd(m, 0x234) & 1 != 0 { 0x60 } else { 0 },
            );
            wr(
                &mut p[1],
                0,
                ty * 0xc0 + if rd(m, 0x238) & 1 != 0 { 0x60 } else { 0 },
            );
            let j = |v: Vec<String>| {
                if v.is_empty() {
                    "-".to_string()
                } else {
                    v.join("/")
                }
            };
            for v in [l0, l1, l2, dx, dy] {
                log.push(j(v));
            }
            0
        }
    });
}
