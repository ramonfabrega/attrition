//! `Unit::add_repair_order@005e4ff0` under the emulator against the port (item 3219; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `Unit::add_repair_order@005e4ff0`, run by `tools/emu/sweep_obj.py` on its row `unit_add_repair_order` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 1024 random bytes of object, 4 argument(s), `p` windows in order: p0..p9 the place(s), in flag order, p10 stub 3's answer window; the global cell 0x00c0618c points at 128 random bytes; the global cell 0x00cae5fc points at 4096 random bytes; the global cell 0x00c0aec0 points at 64 random bytes; the global cell 0x00c06188 points at 320 random bytes; 56792 random bytes at 0x00e3a390; callee 0x005e37f0 (1 args) stubbed; callee 0x005e3920 (0 args) stubbed; callee 0x0060a870 (0 args) stubbed; callee 0x00730ac0 (1 args) stubbed; vtable slot +0x70 (0 args) stubbed; callee 0x0046d5a0 (1 args) stubbed; callee 0x006b52e0 (2 args) stubbed; callee 0x0046f790 (0 args) stubbed; callee 0x0046f290 (0 args) stubbed; the word at `c0+0x14` points at 16 random bytes less `0` (placed); the word at `c0+0x30` points at 16 random bytes less `0` (placed); the word at `p0+0` points at 64 random bytes less `0` (placed); the word at `p0+4` points at 64 random bytes less `0` (placed); the word at `p1+0` points at 64 random bytes less `0` (placed); the word at `p1+4` points at 64 random bytes less `0` (placed); stub 4's answer points at 64 random bytes less `0` (placed); the word at `p6+4` points at 64 random bytes less `0` (placed); the word at `m+0xdc` points at 16 random bytes less `0` (placed); the word at `c3+0x138` points at 4096 random bytes less `0` (placed); stub 6 answered from [0, 1, 2] (90%); stub 7 answered from [0, 1, 2, 3] (90%); stub 8 answered from [1, 0] (90%); stub 3 answers the address of 64 random bytes, a `p` window after the places; the stubbed vtable laid at `p10+0`; stub 6 answers 2 values in call order, printed after the answers; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 5 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 6 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the recorded calls one token a call, in call order, each headed `sIDX:` (the callee); stub 6 prints the 4 bytes its argument 0 points at, hex, after the record (`/` a call); stub 6 prints the 4 bytes its argument 1 points at, hex, after the record (`/` a call); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 1000 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("unit_add_repair_order") else {
        return;
    };
    let obj = super::Obj {
        name: "add_repair_order",
        full: "Unit::add_repair_order@005e4ff0",
        cells: 5,
        calls: true,
        eax: false,
        min_rows: 1000,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, log| {
        let _ = (&a, &m, &c, &p);
        {
            let off = rd(&p[7], 4) as usize;
            let calls = &mut *log;
            if a[2] == 2 {
                wr(m, 0x68, rd(m, 0x68) & 0xfbffffffu32 as i32);
                wr(m, 0xc0, 0);
                calls.push("s0:m+0x0,0".to_string());
                calls.push("s1:m+0x0".to_string());
                calls.push("s2:m+0x0".to_string());
            }
            wr(m, 0x68, rd(m, 0x68) | 0x400);
            wr(&mut p[6], 8, a[0]);
            wr(&mut p[6], 0xc, a[1]);
            let o = 2 + 2 * a[1] as usize + a[0] as usize;
            let short = if a[0] < 0 || a[1] < 0 {
                0xffffu16
            } else {
                u16::from_le_bytes([p[o][0x30], p[o][0x31]])
            };
            p[6][0x10..0x12].copy_from_slice(&short.to_le_bytes());
            if a[3] == 0 {
                p[6][off + 8] &= 0xfb;
            } else {
                p[6][off + 8] |= 4;
            }
            let tile = |c1: &[u8], v: i32| rd(c1, 4 * ((v ^ 0x63637) >> 6) as usize);
            let (ux, uy) = (tile(&c[1], rd(m, 0x10)), tile(&c[1], rd(m, 0x14)));
            let (ox, oy) = (tile(&c[1], rd(&p[o], 0x10)), tile(&c[1], rd(&p[o], 0x14)));
            calls.push("s6:c3+0x0,sp-0x10,sp-0x8".to_string());
            calls.push("s6:c3+0x0,sp-0x4,sp-0xc".to_string());
            let ry = rd(&c[1], 4 * ((rd(&p[o], 0x14) ^ 0x63637) >> 6) as usize);
            let rx = rd(&c[1], 4 * ((rd(&p[o], 0x10) ^ 0x63637) >> 6) as usize);
            let cell = (ry.wrapping_mul(rd(&c[3], 0x18)).wrapping_add(rx)).wrapping_mul(2) as usize;
            if a[14] != a[13] && p[9][cell] & 0x30 != 0x20 {
                let lead = rd(&c[4], m[9] as usize * 0x6eec);
                let tier = if lead & 0x100 != 0 {
                    3
                } else if lead & 0x200 != 0 {
                    2
                } else {
                    (lead as u32 >> 10 & 1) as i32
                };
                if a[11] <= tier && a[12] != 0 {
                    wr(m, 0x68, rd(m, 0x68) | 0x800000);
                }
            }
            calls.push(format!("s5:m+0xcc,p6+0x{:x}", 4 + off));
            calls.push("s2:m+0x0".to_string());
            let hex = |v: i32| {
                v.to_le_bytes()
                    .iter()
                    .map(|b| format!("{b:02x}"))
                    .collect::<String>()
            };
            calls.push(format!("{}/{}", hex(ox), hex(ux)));
            calls.push(format!("{}/{}", hex(oy), hex(uy)));
            0
        }
    });
}
