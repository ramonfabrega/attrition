//! `Unit::build_done@00603bf0` under the emulator against the port (item 3219; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `Unit::build_done@00603bf0`, run by `tools/emu/sweep_obj.py` on its row `unit_build_done` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 336 random bytes of object, 3 argument(s), `p` windows in order: p0..p3 sparse table 0 (2 rows × 2 slice(s), row-major), p4..p15 the place(s), in flag order; the global cell 0x00c0618c points at 128 random bytes; the global cell 0x00c061ec points at 64 random bytes; the global cell 0x00c061f0 points at 64 random bytes; the global cell 0x00c06210 points at 688 random bytes; callee 0x00603e20 (0 args) stubbed; callee 0x00604320 (0 args) stubbed; callee 0x005f5170 (3 args) stubbed; callee 0x0061a5c0 (3 args) stubbed; callee 0x00630450 (2 args) stubbed; vtable slot +0xf8 (0 args) stubbed; vtable slot +0xac (0 args) stubbed; vtable slot +0x90 (0 args) stubbed; vtable slot +0x144 (2 args) stubbed; vtable slot +0x10 (4 args) stubbed; the word at `c0+0x14` points at 16 random bytes less `0` (placed); the word at `p4+0` points at 32 random bytes less `0` (placed); the word at `p4+4` points at 32 random bytes less `0` (placed); the word at `p4+8` points at 32 random bytes less `0` (placed); the word at `p4+12` points at 32 random bytes less `0` (placed); the word at `c0+0x30` points at 16 random bytes less `0` (placed); the word at `p9+0` points at 32 random bytes less `0` (placed); the word at `p9+4` points at 32 random bytes less `0` (placed); the word at `p9+8` points at 32 random bytes less `0` (placed); the word at `p9+12` points at 32 random bytes less `0` (placed); stub 6's answer points at 256 random bytes less `0` (placed); the word at `p14+0x18` points at 32 random bytes less `0` (placed); stub 0 answered from [0, 0, 0, 1] (90%); stub 1 answered from [0, 0, 0, 1] (90%); stub 2 answered from [0, 0, 1] (90%); stub 7 answered from [1, 1, 1, 0] (90%); stub 8 answered from [1, 1, 1, 0] (90%); stub 5 answered from [0, 1, 2] (90%); stub 8 answers 2 values in call order, printed after the answers; stub 5 answers 6 values in call order, printed after the answers; stub 4 answers -1..6 every row; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 3 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 9 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the stack 64 bytes under the call is zeroed before each row; 2 rows 0x9f0 apart at `0xe8d444`, slices +0x0:0x4, +0xc:0x4 (`p` windows after the pointed buffers, before the places); the stubbed vtable laid at `p14+0` as well as at `this`; the stubbed vtable laid at `p15+0` as well as at `this`; the stubbed vtable laid at `p0+0` as well as at `this`; the stubbed vtable laid at `p2+0` as well as at `this`; the stubbed vtable laid at `p5+0` as well as at `this`; the stubbed vtable laid at `p6+0` as well as at `this`; the stubbed vtable laid at `p7+0` as well as at `this`; the stubbed vtable laid at `p8+0` as well as at `this`; the stubbed vtable laid at `p10+0` as well as at `this`; the stubbed vtable laid at `p11+0` as well as at `this`; the stubbed vtable laid at `p12+0` as well as at `this`; the stubbed vtable laid at `p13+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 3000 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("unit_build_done") else {
        return;
    };
    let obj = super::Obj {
        name: "build_done",
        full: "Unit::build_done@00603bf0",
        cells: 4,
        calls: true,
        eax: false,
        min_rows: 3000,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, log| {
        let _ = (&a, &m, &c, &p);
        {
            // Args `a[0..3]` the building, its player and `param_3`; singles `a[3 + stub]`: stub 0
            // `find_build_spot`, 1 `find_repair_spot`, 2 `find_gather_spot`, 4 `num_gatherers`, 7 the
            // type's `+0x90`; `a[13..15]` the record's `+0x144` (`is`) in call order and `a[15..21]` slot
            // `+0xf8` (the unit's worker stance) in call order. Windows: c1 the 0xc061ec cell (byte
            // `+0x2d` the starting resources), c2 `constants` (`+0x28` the gather respond range), c3 the
            // console (`+0x2a0` the player), p0..p3 the select groups' (vtable word, count) per player,
            // p14 the record (`+0x80` its signed `gather_max`). `find_gather_spot`'s third argument is the
            // residue of the last return address, the one below its two unwritten slots.
            let mut f8n = 0usize;
            let mut f8 = || {
                f8n += 1;
                a[15 + (f8n - 1).min(5)]
            };
            let mut toks = [
                "-".to_string(),
                "-".into(),
                "-".into(),
                "-".into(),
                "-".into(),
            ];
            let who = m[9];
            let ty = i16::from_le_bytes([m[0xa], m[0xb]]);
            'done: {
                if rd(m, 0xd8) > 1 {
                    break 'done;
                }
                if rd(m, 0x68) & 0x40000 != 0 {
                    toks[0] = "m+0x0".into();
                    if a[3] != 0 {
                        break 'done;
                    }
                    toks[1] = "m+0x0".into();
                    if a[4] != 0 {
                        break 'done;
                    }
                    if c[1][0x2d] == 8 {
                        break 'done;
                    }
                    toks[2] = format!("m+0x0,{},0,{}", a[2], 0x603c22);
                    break 'done;
                }
                let v1 = f8();
                if v1 == 1 || f8() == 2 {
                    toks[0] = "m+0x0".into();
                    if a[3] != 0 {
                        break 'done;
                    }
                }
                let w = f8();
                let (gather, ret_addr) = if w == 1 {
                    (true, 0x603c82)
                } else {
                    (f8() == 0, 0x603c91)
                };
                if gather {
                    toks[2] = format!(
                        "m+0x0,{},0,{}",
                        rd(&c[2], 0x28).wrapping_mul(0xc0),
                        ret_addr
                    );
                    if a[5] != 0 {
                        m[0x80..0x82].copy_from_slice(&0xffffu16.to_le_bytes());
                        let play = rd(&c[3], 0x2a0) as usize;
                        if rd(&p[2 * play + 1], 0) >= 2 {
                            toks[4] = format!("p{}+0x0,{ty},{who},0,0", 2 * play);
                        }
                        break 'done;
                    }
                }
                let x = f8();
                if x == 1 || f8() == 2 {
                    toks[1] = "m+0x0".into();
                    if a[4] != 0 {
                        break 'done;
                    }
                }
                if a[0] < 0 || i32::from(who) != a[1] {
                    break 'done;
                }
                if a[10] == 0 {
                    break 'done;
                }
                if a[13] == 0 {
                    break 'done;
                }
                if a[14] == 0 {
                    break 'done;
                }
                let max = i32::from(p[14][0x80] as i8);
                if a[7] < max {
                    toks[3] = format!("m+0x0,{},2,0", a[0]);
                }
            }
            log.extend(toks);
            0
        }
    });
}
