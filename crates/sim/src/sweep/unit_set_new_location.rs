//! `Unit::set_new_location@005f8d20` under the emulator against the port (item 3289; tools/sweep_gen.py, obj with windows).

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

/// `Unit::set_new_location@005f8d20`, run by `tools/emu/sweep_unit_set_new_location.py` under unicorn
/// (tools/emu): 256 random bytes of object, 4 argument(s), `p` windows in order: p0..p13 the place(s), in flag order; the global cell 0x00cae5fc points at 1024 random bytes; the global cell 0x00c06188 points at 320 random bytes; the global cell 0x00c06218 points at 16 random bytes; vtable slot +0xbc (0 args) stubbed; callee 0x006e1370 (1 args) stubbed; callee 0x005e4a60 (7 args) stubbed; callee 0x0064cd20 (4 args) stubbed; callee 0x00646d50 (1 args) stubbed; vtable slot +0x158 (3 args) stubbed; callee 0x00647080 (3 args) stubbed; callee 0x00647970 (0 args) stubbed; callee 0x008544a0 (4 args) stubbed; callee 0x0064d8c0 (0 args) stubbed; vtable slot +0x30 (0 args) stubbed; callee 0x005f9780 (0 args) stubbed; vtable slot +0x174 (1 args) stubbed; callee 0x005f96b0 (4 args) stubbed; callee 0x005d9010 (2 args) stubbed; callee 0x005d86f0 (3 args) stubbed; callee 0x00a46a00 (0 args) run, not hooked; the word at `m+0xf4` points at 40 random bytes less `0` (placed); the word at `p0+0` points at 128 random bytes less `0` (placed); the word at `p0+4` points at 128 random bytes less `0` (placed); the word at `p0+8` points at 128 random bytes less `0` (placed); the word at `p0+12` points at 128 random bytes less `0` (placed); the word at `p0+16` points at 128 random bytes less `0` (placed); the word at `p0+20` points at 128 random bytes less `0` (placed); the word at `p0+24` points at 128 random bytes less `0` (placed); the word at `p0+28` points at 128 random bytes less `0` (placed); the word at `p0+32` points at 128 random bytes less `0` (placed); the word at `p0+36` points at 128 random bytes less `0` (placed); the word at `m+0x18` points at 768 random bytes less `0` (placed); the word at `c1+0x134` points at 2048 random bytes less `0` (placed); the word at `c1+0x138` points at 1280 random bytes less `0` (placed); slot +0xbc laid as 0x0046ce30 or its stub, a flag after the answers; slot +0x158 laid as 0x0060eda0 or its stub, a flag after the answers; stub 0 answered from [0, 1] (90%); stub 1 answered from [0, 1] (90%); stub 4 answered from [0, 1, 2] (90%); stub 10 answered from [0, 0, 1] (90%); stub 0 answers 2 values in call order, printed after the answers; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 3 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 4 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 5 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 6 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 7 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 8 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 9 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 10 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 11 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 12 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 13 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 14 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 15 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the recorded calls one token a call, in call order, each headed `sIDX:` (the callee); stub 8 writes a value in -5000..5000 through its argument 0, printed after the answers (`o` in a place); `0x00a46980` run once before the rows (a table builder the startup would have run); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 3000 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_unit_set_new_location.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (line, calls) = line.split_once(" # ").expect("`<row> # <calls>`");
        let want_log: Vec<&str> = calls.split_whitespace().collect();
        let (lhs, rhs) = line
            .split_once(" -> ")
            .expect("a row is `<args…> | <windows> -> <eax> | <windows>`");
        let (args, before) = lhs.split_once(" | ").expect("`<args> | <before>`");
        let (eax, after) = rhs.split_once(" | ").expect("`<eax> | <after>`");
        let a: Vec<i32> = args
            .split_whitespace()
            .map(|s| s.parse().expect("an integer"))
            .collect();
        let mut wins: Vec<Vec<u8>> = before.trim().split(';').map(unhex).collect();
        let mut m = wins.remove(0);
        let mut p: Vec<Vec<u8>> = wins.split_off(3);
        let mut c: Vec<Vec<u8>> = wins;
        let want_eax: i64 = eax.trim().parse().expect("an integer");
        let mut log: Vec<String> = Vec::new();
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            let _ = (&a, &m, &c, &p);
            {
                // **No crate method exists for this function**: the port spreads it over `collide.rs`'s
                // `Sim::set_new_location` (the move, the shore half, the goody box), `lib.rs`'s `crew_des`
                // (guy 0 and the tracked crew), `vision::moved_to` (the half-cell test) and `transport.rs`
                // (the transport spells). It has no `_steps` over a view, and it does not model the squad
                // arm at all (`guy_mark` is 1 on every dumped unit), the Iroquois `unit_masks & 0x800`
                // write, or `update_ceo_position` and the `+0x174` slot. So this block is an inline copy,
                // read from the listing, over the same bytes; it calls the crate's own `sin_component`,
                // `fog_of` and the world-cell constants where the crate has them.
                //
                // `a`: 0 `x`, 1 `y`, 2 `param_3` (move the guys), 3 `param_4` (turn them); then stub
                // k's answer at `4 + k`, 21 what `find_tcoord_z` writes through its first argument, 22
                // and 23 the `+0xbc` slot's first and second answers, 24 whether `+0xbc` is laid as
                // `UnitData::is_on_map`, 25 whether `+0x158` is laid as `Object::die`.
                // Windows: `c[0]` the `div_3_table`, `c[1]` the world record, `c[2]` the terrain cell's
                // record, `p[0]` the guys' pointer array, `p[1..=10]` the guys, `p[11]` the type record,
                // `p[12]` the world's `+0x134` table, `p[13]` its `+0x138` table.
                use crate::movement::{Angle, sin_component};
                use crate::vision::fog_of;
                use crate::world::{UNITS_PER_CELL, UNITS_PER_TILE};
                const KEY: i32 = 0x63637;
                fn rd32(w: &[u8], o: usize) -> i32 {
                    i32::from_le_bytes(w[o..o + 4].try_into().expect("four bytes"))
                }
                fn wr32(w: &mut [u8], o: usize, v: i32) {
                    w[o..o + 4].copy_from_slice(&v.to_le_bytes());
                }
                fn rd16(w: &[u8], o: usize) -> u16 {
                    u16::from_le_bytes(w[o..o + 2].try_into().expect("two bytes"))
                }
                fn cell(v: i32) -> i32 {
                    v.div_euclid(UNITS_PER_CELL)
                }
                fn tile(v: i32) -> i32 {
                    v.div_euclid(UNITS_PER_TILE)
                }
                fn half(v: i32) -> i32 {
                    (v + i32::from(v < 0)) >> 1
                }
                fn model(
                    a: &[i32],
                    m: &mut [u8],
                    c: &[Vec<u8>],
                    p: &mut [Vec<u8>],
                    calls: &mut Vec<String>,
                ) -> i32 {
                    let _ = c;
                    let (x, y, p3, p4) = (a[0], a[1], a[2], a[3]);
                    let ans = |k: usize| a[4 + k];
                    let (out_z, seq, is_on_map_base, die_base) =
                        (a[21], [a[22], a[23]], a[24] == 1, a[25] == 1);
                    let this = "m+0x0";
                    let (ox, oy) = (rd32(m, 0x10) ^ KEY, rd32(m, 0x14) ^ KEY);
                    let (otx, oty) = (tile(ox), tile(oy));
                    let (ocx, ocy) = (cell(ox), cell(oy));
                    let mut asked = 0usize;
                    let mut on_map = |m: &[u8]| -> bool {
                        let r = if is_on_map_base {
                            i32::from(rd16(m, 0x82) >> 15)
                        } else {
                            asked += 1;
                            seq[asked - 1]
                        };
                        r != 0
                    };
                    let cell_change = on_map(m) && !(ocx == cell(x) && ocy == cell(y));
                    let tile_change = on_map(m) && !(otx == tile(x) && oty == tile(y));
                    let ty218 = rd32(&p[11], 0x218);
                    let (w0, w4, w18) = (rd32(&c[1], 0), rd32(&c[1], 4), rd32(&c[1], 0x18));
                    let tile_bits = |p: &[Vec<u8>]| -> u16 {
                        rd16(
                            &p[13],
                            (2 * w18.wrapping_mul(tile(y)).wrapping_add(tile(x))) as usize,
                        )
                    };
                    let mut enter = cell_change;
                    if !cell_change && tile_change {
                        let bit = (rd16(&p[12], (0x1c * (w0 * ocy + ocx)) as usize) >> 8) & 1;
                        let owner = i32::from(m[9]);
                        calls.push(format!("s1:{},18", 0x00e3_a390 + owner * 0x6eec));
                        if ans(1) != 0 && ty218 == 0 {
                            let flags = rd32(m, 0x68);
                            let on = tile_bits(p) & 0x30 == 0x30;
                            wr32(m, 0x68, if on { flags | 0x800 } else { flags & !0x800 });
                        }
                        enter = bit != 0;
                    }
                    if enter {
                        let (f68, f6c) = (rd32(m, 0x68), rd32(m, 0x6c));
                        if (f68 & 0x800000 != 0 && f6c & 0x2000 == 0) || p[11][0x2b4] & 0x10 != 0 {
                            if ty218 == 0 && tile_bits(p) & 0x30 == 0x20 {
                                calls.push(format!("s2:{this},-1,-1,-1,-1,650,0,0"));
                                return 0;
                            }
                            if ty218 == 1 && tile_bits(p) & 0x30 != 0x20 {
                                calls.push(format!("s3:{this},0,-1,1,1"));
                                calls.push(format!("s4:{this},1"));
                                if ans(4) == 0 {
                                    let slot = if die_base { 6 } else { 5 };
                                    calls.push(format!("s{slot}:{this},0,-1,0"));
                                }
                                return 0;
                            }
                        }
                        if cell_change {
                            calls.push(format!("s7:{this}"));
                        }
                    }
                    wr32(m, 0x10, x ^ KEY);
                    wr32(m, 0x14, y ^ KEY);
                    calls.push(format!("s8:c2+0x0,sp-0x34,{},{},1", tile(x), tile(y)));
                    wr32(m, 0xc, out_z ^ KEY);
                    if cell_change {
                        calls.push(format!("s9:{this}"));
                        calls.push(format!("s10:{this}"));
                        if ans(10) == 0
                            && rd32(m, 0x68) & 1 == 0
                            && ty218 == 0
                            && (rd16(&p[12], (0x1c * (w0 * cell(y) + cell(x))) as usize) as i16) < 0
                        {
                            calls.push(format!("s11:{this}"));
                        }
                    }
                    if tile_change {
                        if (otx >> 1) != fog_of(x) || (oty >> 1) != fog_of(y) {
                            calls.push(format!("s12:{this},{}", i32::from(p3 == 0)));
                        }
                        if rd32(m, 0x6c) & 0x10000 != 0 {
                            calls.push(format!("s13:{this},{otx},{oty},{},{}", tile(x), tile(y)));
                        }
                    }
                    let n = i32::from(m[0xb5] as i8);
                    let angle = rd32(m, 0x50);
                    let (nx, ny) = (rd32(m, 0x10) ^ KEY, rd32(m, 0x14) ^ KEY);
                    if m[0xb5] == 1 {
                        if p4 != 0 {
                            calls.push(format!("s14:p1+0x0,{angle},{p3}"));
                        }
                        wr32(&mut p[1], 0x5c, nx);
                        wr32(&mut p[1], 0x60, ny);
                        if p3 != 0 {
                            calls.push(format!("s15:p1+0x0,{nx},{ny},1"));
                        }
                        return 1;
                    }
                    let aa = m[0xaa];
                    let mag = (i32::from(aa == 5) + 1).wrapping_mul(rd32(&p[11], 0x224));
                    let mut cosc = if mag == 0 {
                        0
                    } else {
                        sin_component(Angle(angle.wrapping_sub(0x4000_0000)), mag)
                    };
                    let mut sinc = if mag == 0 {
                        0
                    } else {
                        sin_component(Angle(angle), mag)
                    };
                    let per_row = if aa == 8 {
                        i32::from(n != 1) + 1
                    } else {
                        n.min(3)
                    };
                    let q = (n - 1) / per_row;
                    if rd32(m, 0x68) & 2 != 0 {
                        cosc = cosc.wrapping_neg();
                        sinc = sinc.wrapping_neg();
                    }
                    let rows1 = per_row - 1;
                    let off_x =
                        half(q.wrapping_mul(sinc)).wrapping_sub(half(rows1.wrapping_mul(cosc)));
                    let off_y =
                        half(q.wrapping_mul(cosc)).wrapping_add(half(rows1.wrapping_mul(sinc)));
                    for i in 0..n.max(0) {
                        let (row, col) = (i / per_row, i % per_row);
                        let w = 1 + i as usize;
                        if p4 != 0 {
                            calls.push(format!("s14:p{w}+0x0,{angle},{p3}"));
                        }
                        let mut gx = col
                            .wrapping_mul(cosc)
                            .wrapping_sub(row.wrapping_mul(sinc))
                            .wrapping_add(nx)
                            .wrapping_add(off_x);
                        let mut gy = ny
                            .wrapping_sub(col.wrapping_mul(sinc))
                            .wrapping_sub(row.wrapping_mul(cosc))
                            .wrapping_add(off_y);
                        let (lx, ly) = (w0.wrapping_mul(0x300), w4.wrapping_mul(0x300));
                        if gx >= lx {
                            gx = lx - 1;
                        } else if gx < 0 {
                            gx = 0;
                        }
                        if gy >= ly {
                            gy = ly - 1;
                        } else if gy < 0 {
                            gy = 0;
                        }
                        wr32(&mut p[w], 0x5c, gx);
                        wr32(&mut p[w], 0x60, gy);
                        if p3 != 0 {
                            calls.push(format!("s15:p{w}+0x0,{gx},{gy},1"));
                        }
                    }
                    1
                }
                let mut calls = Vec::new();
                let r = model(a, m, c, p, &mut calls);
                if calls.is_empty() {
                    calls.push("-".to_string());
                }
                log.extend(calls);
                i64::from(r)
            }
        };
        assert_eq!(got, want_eax, "set_new_location — eax, row `{line}`");
        assert_eq!(
            log, want_log,
            "set_new_location — the recorded calls, row `{line}`"
        );
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "set_new_location — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(
                w,
                &want[1 + i],
                "set_new_location — window c[{i}], row `{line}`"
            );
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(
                w,
                &want[4 + i],
                "set_new_location — window p[{i}], row `{line}`"
            );
        }
        n += 1;
    }
    assert!(n >= 3000, "only {n} rows");
    eprintln!("Unit::set_new_location@005f8d20: {n} rows agree");
}
