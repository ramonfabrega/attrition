//! `Borders::save_last_who@008835e0` under the emulator against the port (item 3038; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `Borders::save_last_who@008835e0`, run by `tools/emu/sweep_obj.py` on its row `borders_save_last_who` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 8 random bytes of object, 4 argument(s), `p` windows in order: p0..p5 the place(s), in flag order; the global cell 0x00c0621c points at 1024 random bytes; the global cell 0x00c061d0 points at 1024 random bytes; callee 0x00882300 (1 args) stubbed; callee 0x004a0cb0 (1 args) stubbed; the word at `c0+0x10` points at 12 random bytes less `0` (placed); the word at `p0+0` points at 128 random bytes less `0` (placed); the word at `p0+4` points at 128 random bytes less `0` (placed); the word at `p0+8` points at 128 random bytes less `0` (placed); the word at `c0+0x2c` points at 512 random bytes less `0` (placed); the word at `c1+0x134` points at 1824 random bytes less `0` (placed); stub 0 answered from [0, 1, 1] (90%); stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 prints the 2 bytes its argument 0 points at, hex, after the record (`/` a call); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 2500 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("borders_save_last_who") else {
        return;
    };
    let obj = super::Obj {
        name: "save_last_who",
        full: "Borders::save_last_who@008835e0",
        cells: 2,
        calls: true,
        eax: false,
        min_rows: 2500,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, log| {
        let _ = (&a, &m, &c, &p);
        // The borders cell's 8-byte cells (table at +0x2c, 16-bit index `width(+0x224) * y + x`) get
        // the two sides' marks in their flag byte (+1): 0x40 for the tile's own owner (the tile record's
        // signed byte +0xf, row-major in the world cell), 0x80 for `a[2]` (argument 2, who), when the
        // sides differ and `a[3]` is non-zero; both sides hidden (the stub, one answer for both calls)
        // is a return. A mark on an unmarked cell (flag byte & 0xc0 == 0) adds the index to the
        // `a[3]`-th border's list (stub 1, `ecx` = that record + 0x58).
        let (x, y, who, side) = (a[0], a[1], a[2], a[3]);
        let hidden = a[4] != 0;
        let width = i32::from(u16::from_le_bytes([c[0][0x224], c[0][0x225]]));
        let idx = ((width * y + x) as u16) as usize;
        let tile = ((rd(&c[1], 0) * y + x) * 0x1c) as usize;
        let esi = i32::from(p[5][tile + 0xf] as i8).max(-1);
        let edi = who.max(-1);
        let (mut adds, mut derefs) = (Vec::new(), Vec::new());
        'body: {
            if edi >= 0 && esi >= 0 && hidden {
                break 'body;
            }
            let at = idx * 8 + 1;
            let old = p[4][at];
            let edx = old & 0xc0;
            if side == 0 || edi == esi {
                break 'body;
            }
            let marked = esi >= 0;
            if marked {
                p[4][at] = old | 0x40;
            }
            if edi >= 0 {
                p[4][at] |= 0x80;
            } else if !marked {
                break 'body;
            }
            if edx == 0 {
                adds.push(format!("p{side}+0x58,sp-0x10"));
                derefs.push(format!("{:02x}{:02x}", idx & 0xff, idx >> 8));
            }
        }
        log.push(if adds.is_empty() {
            "-".into()
        } else {
            adds.join("/")
        });
        log.push(if derefs.is_empty() {
            "-".into()
        } else {
            derefs.join("/")
        });
        0
    });
}
