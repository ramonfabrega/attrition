//! `Animal::work@005d7330` under the emulator against the port (item 3387; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `Animal::work@005d7330`, run by `tools/emu/sweep_obj.py` on its row `animal_work` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 256 random bytes of object, 0 argument(s), `p` windows in order: p0..p7 sparse table 0 (8 rows × 1 slice(s), row-major), p8..p27 the place(s), in flag order; the global cell 0x00c061ec points at 1376 random bytes; the global cell 0x00cae5fc points at 256 random bytes; the global cell 0x00c06188 points at 16 random bytes; vtable slot +0x10 (0 args) stubbed; callee 0x00617a10 (2 args) stubbed; callee 0x006b55c0 (2 args) stubbed; the word at `0xc0a250` points at 28 random bytes less `0` (placed); the word at `p8+0` points at 24 random bytes less `0` (placed); the word at `p8+4` points at 24 random bytes less `0` (placed); the word at `p8+8` points at 24 random bytes less `0` (placed); the word at `p8+12` points at 24 random bytes less `0` (placed); the word at `p8+16` points at 24 random bytes less `0` (placed); the word at `p8+20` points at 24 random bytes less `0` (placed); the word at `p8+24` points at 24 random bytes less `0` (placed); the word at `0xc0a0f0` points at 32 random bytes less `0` (placed); the word at `p16+0` points at 36 random bytes less `0` (placed); the word at `p16+4` points at 36 random bytes less `0` (placed); the word at `p16+8` points at 36 random bytes less `0` (placed); the word at `p16+12` points at 36 random bytes less `0` (placed); the word at `p16+16` points at 36 random bytes less `0` (placed); the word at `p16+20` points at 36 random bytes less `0` (placed); the word at `p16+24` points at 36 random bytes less `0` (placed); the word at `p16+28` points at 36 random bytes less `0` (placed); the word at `m+0xdc` points at 8 random bytes less `0` (placed, null a third of the rows); the word at `p25+4` points at 16 random bytes less `0` (placed); the word at `p26+8` points at 32 random bytes less `0` (placed, null a third of the rows); the stubbed vtable laid at `p27+0`; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the recorded calls one token a call, in call order, each headed `sIDX:` (the callee); stub 2 prints the 4 bytes its argument 0 points at, hex, after the record (`/` a call); stub 2 prints the 4 bytes its argument 1 points at, hex, after the record (`/` a call); 8 rows 0x6eec apart at `@0xe3a390`, slices +0x0:0x1 (`p` windows after the pointed buffers, before the places); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 800 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("animal_work") else {
        return;
    };
    let obj = super::Obj {
        name: "work",
        full: "Animal::work@005d7330",
        cells: 3,
        calls: true,
        eax: false,
        min_rows: 800,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, log| {
        let _ = (&a, &m, &c, &p);
        // hand: port
        // The crate keeps no goods seen-mask and no order cursor for an animal (its `work` and
        // `Unit::work` are namesakes, `Animal::work` being the animal's own entry), so the
        // steps are ported inline: the order cursor, one `do_job`, then the herd's seen marks.
        // A window's pointer is a word the port cannot see through; `p25` is the cursor node,
        // `p26` its record, `p27` the order, `p8` the herd table, `p17..` the goods records.
        let sh = |w: &[u8], o: usize| i16::from_le_bytes([w[o], w[o + 1]]);
        let mut order = 0;
        let mut cc = 0;
        if rd(m, 0xdc) != 0 {
            let d4 = rd(&p[25], 4);
            m[0xd4..0xd8].copy_from_slice(&d4.to_le_bytes());
            cc = rd(&p[26], 8);
            m[0xcc..0xd0].copy_from_slice(&cc.to_le_bytes());
            m[0xd0] = p[26][0xc];
            if cc != 0 {
                log.push("s0:p27+0x0".to_string());
                order = a[0];
            }
        }
        log.push(format!(
            "s1:m+0x0,{order},{}",
            if cc != 0 { "p27+0x0" } else { "0" }
        ));
        let herd = i32::from(sh(m, 0x86));
        let mut arg0: Vec<[u8; 4]> = Vec::new();
        let mut arg1: Vec<[u8; 4]> = Vec::new();
        if herd != -1 {
            let tick = i32::from(sh(m, 0xa)).wrapping_add(rd(&c[0], 0x550));
            if tick % 16 == 0 {
                let good = rd(&p[(9 + herd) as usize], 0x14);
                if good != -1 {
                    let leader = if m[9] < 8 { p[usize::from(m[9])][0] } else { 0 };
                    for j in 0..8 {
                        if leader & 1 != 0 {
                            let tab = |v: i32| rd(&c[1], 4 * ((v ^ 0x63637) >> 7) as usize);
                            // arg 0 is the 0x10 word's local, arg 1 the 0x14 word's.
                            arg0.push(tab(rd(m, 0x10)).to_le_bytes());
                            arg1.push(tab(rd(m, 0x14)).to_le_bytes());
                            log.push("s2:c2+0x0,sp-0x10,sp-0xc".to_string());
                            if a[2] != 0 {
                                p[(17 + good) as usize][0x20] |= 1 << j;
                            }
                        }
                    }
                }
            }
        }
        let hex = |v: &[[u8; 4]]| -> String {
            v.iter()
                .map(|b| b.iter().map(|x| format!("{x:02x}")).collect::<String>())
                .collect::<Vec<_>>()
                .join("/")
        };
        if arg0.is_empty() {
            log.push("-".to_string());
            log.push("-".to_string());
        } else {
            log.push(hex(&arg0));
            log.push(hex(&arg1));
        }
        0
        // end hand
    });
}
